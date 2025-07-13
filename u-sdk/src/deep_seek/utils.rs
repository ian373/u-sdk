use super::types::{Message, Role};
use crate::deep_seek::Error;

pub(crate) fn check_msg_list(msg_list: &[Message]) -> Result<(), Error> {
    // 多轮对话的形式

    if msg_list.is_empty() {
        return Err(Error::Common("Message list cannot be empty".to_string()));
    } else if msg_list.last().unwrap().role != Role::User {
        // 最后一条消息必须是 User
        return Err(Error::Common(
            "The last message must be from the user".to_string(),
        ));
    }

    Ok(())
}
