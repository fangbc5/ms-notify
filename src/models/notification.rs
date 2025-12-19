use super::ChannelType;
use serde::{Deserialize, Serialize};

/// 通知消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// 发送者（邮件时使用）
    pub from: String,
    /// 接收者（邮件、短信时使用）
    pub to: String,
    /// 主题（邮件时使用）
    pub subject: String,
    /// 消息内容
    pub body: String,
    /// 消息渠道类型
    pub channel: ChannelType,
}
