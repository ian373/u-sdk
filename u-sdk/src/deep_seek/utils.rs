use super::types::{Message, Role};

pub(crate) fn check_msg_list(msg_list: &[Message]) -> Result<(), String> {
    // 多轮对话的形式

    if msg_list.is_empty() {
        return Err("msg_list is empty".to_string());
    } else if msg_list.last().unwrap().role != Role::User {
        // 最后一条消息必须是 User
        return Err("The last message role must be User".to_string());
    }

    Ok(())
}
