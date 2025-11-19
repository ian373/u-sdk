use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::{Engine, engine::general_purpose::STANDARD};
use md5::{Digest, Md5};
// use percent_encoding::percent_decode_str;
use rsa::pkcs8::DecodePublicKey;
use rsa::signature::hazmat::PrehashVerifier;
use rsa::{RsaPublicKey, pkcs1v15};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Debug, thiserror::Error)]
enum OssVerifyError<'a> {
    #[error("missing required header `{0}`")]
    MissingHeader(&'a str),

    #[error("invalid header `{0}`")]
    InvalidHeader(&'a str),

    #[error("invalid oss callback signature")]
    InvalidSignature,

    #[error("failed to read request body: {0}")]
    BodyRead(#[from] axum::Error),

    #[error("http error when verifying oss public key: {0}")]
    Http(#[from] reqwest::Error),

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("utf-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("error: {0}")]
    Common(&'a str),
}

impl IntoResponse for OssVerifyError<'_> {
    fn into_response(self) -> Response {
        match self {
            OssVerifyError::MissingHeader(name) => (
                StatusCode::BAD_REQUEST,
                format!("missing required header `{name}`"),
            )
                .into_response(),

            OssVerifyError::InvalidHeader(name) => {
                (StatusCode::BAD_REQUEST, format!("invalid header `{name}`")).into_response()
            }

            OssVerifyError::InvalidSignature => {
                (StatusCode::BAD_REQUEST, "invalid oss callback signature").into_response()
            }

            OssVerifyError::BodyRead(e) => (
                StatusCode::BAD_REQUEST,
                format!("failed to read request body: {e}"),
            )
                .into_response(),

            OssVerifyError::Http(e) => (
                StatusCode::BAD_GATEWAY,
                format!("http error when verifying oss public key: {e}"),
            )
                .into_response(),

            OssVerifyError::Base64(e) => {
                (StatusCode::BAD_REQUEST, format!("base64 decode error: {e}")).into_response()
            }

            OssVerifyError::Utf8(e) => {
                (StatusCode::BAD_REQUEST, format!("utf-8 error: {e}")).into_response()
            }

            OssVerifyError::Common(msg) => {
                (StatusCode::BAD_REQUEST, format!("error: {msg}")).into_response()
            }
        }
    }
}

/// OSS回调验证成功后的Body的数据
///
/// 验证成功后，存放oss发过来的body数据，为application/json或application/x-www-form-urlencoded(具体视调用callback api时的设置而定)。
///
/// 在axum中可以把它写为一个extractor，方便handler直接使用。下面给一个构建提取json的例子：
/// ```rust,no_run
/// use axum::{
///     extract::FromRequestParts,
///     http::{StatusCode, request::Parts},
///     response::{IntoResponse, Response},
/// };
/// use serde::de::DeserializeOwned;
///
/// #[derive(Debug, Clone)]
/// pub struct VerifiedOssCallbackBody(pub String);
///
/// #[derive(Debug)]
/// pub struct VerifiedOssJson<T>(pub T);
///
/// impl<S, T> FromRequestParts<S> for VerifiedOssJson<T>
/// where
///     S: Send + Sync,
///     T: DeserializeOwned,
/// {
///     type Rejection = Response;
///
///     async fn from_request_parts(
///         parts: &mut Parts,
///         _state: &S,
///     ) -> Result<Self, Self::Rejection> {
///         // 1. 从 extensions 里拿到之前中间件塞进去的 VerifiedOssCallbackBody
///         let ext = parts
///             .extensions
///             .get::<VerifiedOssCallbackBody>()
///             .ok_or_else(|| {
///                 (
///                     StatusCode::INTERNAL_SERVER_ERROR,
///                     "VerifiedOssCallbackBody missing",
///                 )
///                     .into_response()
///             })?;
///
///         // 2. 把里面的 String 按 JSON 解析成 T
///         let value = serde_json::from_str::<T>(&ext.0).map_err(|e| {
///             (
///                 StatusCode::BAD_REQUEST,
///                 format!("invalid oss callback json: {e}"),
///             )
///                 .into_response()
///         })?;
///
///         Ok(VerifiedOssJson(value))
///     }
/// }
///
/// // 然后在handler里就可以直接用VerifiedOssJson<T>来接收解析后的数据：
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// struct OssCallbackPayload {
///     // 按你的业务字段来
///     pub user_id: String,
///     pub filename: String,
///     pub size: u64,
/// }
///
/// // 使用 VerifiedOssJson 提取器：
/// async fn oss_callback(
///     VerifiedOssJson(payload): VerifiedOssJson<OssCallbackPayload>,
/// ) -> impl IntoResponse {
///     dbg!(&payload);
///     "ok"
/// }
/// ```
#[derive(Debug, Clone)]
pub struct VerifiedOssCallbackBody(pub String);

/// 只支持 tokio + axum
/// 验证成功会把oss发过来的body以String形式放在extensions里：[VerifiedOssCallbackBody]
#[derive(Clone)]
pub struct OssCallbackVerifyLayer {
    client: reqwest::Client,
    // 这里用 Arc 包一层，便于不同 Service 共享缓存
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    // TODO 因为目前配置的nginx/axum会把url路径给剥离掉，所以先用这个字段让用户自己填上回调路径
    callback_path: String,
}

impl OssCallbackVerifyLayer {
    /// callback_url_path: 在设置`callbackUrl`时的路径部分
    ///
    /// 因为如果应用部署在代理如nginx后面，nginx可能会配置把路径前缀剥离掉；
    /// 或者axum如果是嵌套路由，那么在layer的service里看到的uri.path()也不是完整路径，
    /// 此时需要使用（如果路径没有被nginx等剥离）[axum::extract::OriginalUri]来获取完整路径。
    /// 为了简化起见，这里直接让用户传入callbackUrl里的路径部分
    ///
    /// 注意：传入的是没有经过url encode的原始路径
    pub fn new(callback_url_path: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            // 也可以直接用自己的 HashMap，这里示范复用全局缓存
            cache: Arc::new(RwLock::new(HashMap::new())),
            callback_path: callback_url_path.to_owned(),
        }
    }

    // 如果你想让多个 Layer / 多 crate 共享同一份缓存，可以用这个构造
    // pub fn with_global_cache() -> Self {
    //     Self {
    //         client: reqwest::Client::new(),
    //         cache: Arc::new(RwLock::new(GLOBAL_PUB_KEY_CACHE.read().unwrap().clone())),
    //     }
    // }
}

impl<S> Layer<S> for OssCallbackVerifyLayer {
    type Service = OssCallbackVerifyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OssCallbackVerifyService {
            inner,
            client: self.client.clone(),
            cache: Arc::clone(&self.cache),
            callback_path: self.callback_path.clone(),
        }
    }
}

/// OssCallbackVerifyLayer对应的Service实现
#[derive(Clone)]
pub struct OssCallbackVerifyService<S> {
    inner: S,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    callback_path: String,
}

// 构建Service的过程tower有一个guide: https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md
impl<S> Service<Request<Body>> for OssCallbackVerifyService<S>
where
    // 这里要求S: Clone是因为我们在call里需要clone它
    // S 必须是一个处理 HTTP 请求的 Service，返回的是 axum 的 Response，
    // 这样这个中间件才能挂在 axum 的 Router / 其它 HTTP 中间件前后
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    // 不强制 S::Error 是具体什么类型，但要求它能转换成 axum::BoxError，
    // 方便和 axum/tower 生态里那些统一用 BoxError 的通用组件（如 HandleErrorLayer）组合
    S::Error: Into<axum::BoxError>,
    // 这个必须要否则会在call返回的Future里报错，如果没有这个，即使你返回的Future是Send的，编译器也会报错
    // axum文档中的例子也有：https://docs.rs/axum/latest/axum/middleware/index.html#towerservice-and-pinboxdyn-future
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // Future 必须是 Send + 'static，因为可能会跨线程（tokio默认是多线程运行时）
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // 不直接使用clone，而是使用mem::replace，[文档](https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services)
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let client = self.client.clone();
        let cache = Arc::clone(&self.cache);
        let callback_path = self.callback_path.clone();

        Box::pin(async move {
            // 先做 OSS 验签，如果失败，直接返回 400 Response
            match verify_oss_request(req, &client, &cache, callback_path).await {
                // 这里我们需要把inner clone出来，不能直接用self.inner，否则此时返回的Future的生命周期就不是'static了，而是和self绑定在一起了
                Ok(verified_req) => inner.call(verified_req).await,
                Err(resp) => Ok(resp.into_response()),
            }
        })
    }
}

/// OSS 验签逻辑：
/// - 成功：返回新的 Request（body 已重建，且 extensions 里挂了 VerifiedOssCallbackBody）
/// - 失败：返回一个 400 Response
async fn verify_oss_request<'a>(
    req: Request<Body>,
    client: &reqwest::Client,
    cache: &Arc<RwLock<HashMap<String, Vec<u8>>>>,
    callback_path: String,
) -> Result<Request<Body>, OssVerifyError<'a>> {
    let (parts, body) = req.into_parts();
    let headers = parts.headers.clone();
    let uri = parts.uri.clone();

    // 1. 读 body
    let body_bytes = to_bytes(body, usize::MAX).await?;
    let body_str = String::from_utf8(body_bytes.to_vec())?;

    // 2. 拿 x-oss-pub-key-url 并解码
    let pub_key_url_b64 = header_required(&headers, "x-oss-pub-key-url")?;
    let pub_key_url_raw = STANDARD.decode(pub_key_url_b64.as_bytes())?;
    let pub_key_url = String::from_utf8(pub_key_url_raw)?;

    if !pub_key_url.starts_with("http://gosspublic.alicdn.com/")
        && !pub_key_url.starts_with("https://gosspublic.alicdn.com/")
    {
        return Err(OssVerifyError::Common("invalid oss public key url"));
    }

    // 3. 公钥 PEM：先查缓存，再必要时 HTTP 拉取
    let pub_key_pem = get_or_fetch_pub_key(&pub_key_url, client, cache).await?;
    let pub_key_pem_str = String::from_utf8(pub_key_pem)?;

    // 4. authorization （签名）Base64 解码
    let auth_b64 = header_required(&headers, "authorization")?;
    let auth_bytes = STANDARD.decode(auth_b64.as_bytes())?;

    // 5. 组装 sign_str = url_decode(path) [+ query] + '\n' + body
    // let raw_path = uri.path();
    // let decoded_path = percent_decode_str(raw_path)
    //     .decode_utf8()
    //     .map_err(|_| OssVerifyError::Common("failed to percent-decode uri path"))?
    //     .into_owned();
    let decoded_path = callback_path;

    let auth_path = match uri.query() {
        Some(q) => format!("{}?{}", decoded_path, q),
        None => decoded_path,
    };

    let auth_str = format!("{}\n{}", auth_path, body_str);

    // 6. MD5(auth_str)
    let mut hasher = Md5::new();
    hasher.update(auth_str.as_bytes());
    let digest = hasher.finalize();

    // 7. RSA(PKCS#1 v1.5, MD5) 验签
    let rsa_pub_key = RsaPublicKey::from_public_key_pem(&pub_key_pem_str)
        .map_err(|_| OssVerifyError::Common("failed to parse oss public key pem"))?;

    let verifying_key = pkcs1v15::VerifyingKey::<Md5>::new(rsa_pub_key);
    let signature = pkcs1v15::Signature::try_from(auth_bytes.as_slice())
        .map_err(|_| OssVerifyError::Common("failed to parse oss signature"))?;

    verifying_key
        .verify_prehash(&digest, &signature)
        .map_err(|_| OssVerifyError::InvalidSignature)?;

    // 8. 验签通过：重建 Request，把 body 塞回去，并在 extensions 里挂一份解析好的 body
    let mut new_req = Request::from_parts(parts, Body::from(body_bytes));
    new_req
        .extensions_mut()
        .insert(VerifiedOssCallbackBody(body_str));

    Ok(new_req)
}

/// 取必需 header
fn header_required<'a>(headers: &HeaderMap, name: &'a str) -> Result<String, OssVerifyError<'a>> {
    let value = headers
        .get(name)
        .ok_or(OssVerifyError::MissingHeader(name))?;

    let s = value
        .to_str()
        .map_err(|_| OssVerifyError::InvalidHeader(name))?;
    Ok(s.to_owned())
}

/// 按“公钥 URL -> PEM”缓存
async fn get_or_fetch_pub_key<'a>(
    url: &str,
    client: &reqwest::Client,
    cache: &Arc<RwLock<HashMap<String, Vec<u8>>>>,
) -> Result<Vec<u8>, OssVerifyError<'a>> {
    // 先查缓存
    {
        let cache_read = cache.read().unwrap();
        if let Some(v) = cache_read.get(url) {
            return Ok(v.clone());
        }
    }

    // 缓存未命中，走 HTTP
    let resp = client.get(url).send().await?;
    let bytes = resp.bytes().await?;

    // 写回缓存
    {
        let mut cache_write = cache.write().unwrap();
        cache_write.insert(url.to_string(), bytes.to_vec());
    }

    Ok(bytes.to_vec())
}
