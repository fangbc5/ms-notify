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
    ///
    /// 根据渠道类型有不同的用途：
    /// - **邮件渠道 (Email)**：作为邮件正文内容，支持纯文本或 HTML 格式
    /// - **短信渠道 (Sms)**：作为 JSON 格式的模板参数字符串，用于填充短信模板变量
    /// - **钉钉渠道 (ImDingding)**：支持两种格式：
    ///   - JSON 对象格式：`{"msg_type": "text|markdown|link|actionCard|...", "content": {...}}`
    ///   - 纯文本格式：直接作为文本消息发送
    /// - **飞书渠道 (ImFeishu)**：支持两种格式：
    ///   - JSON 对象格式：`{"msg_type": "text|post|image|interactive|...", "content": {...}}`
    ///   - 纯文本格式：直接作为文本消息发送
    /// - **企业微信渠道 (ImWechat)**：消息内容（具体格式待实现）
    /// - **推送通知渠道 (Push)**：推送消息内容（具体格式待实现）
    /// - **站内消息渠道 (SiteMessage)**：站内消息内容（具体格式待实现）
    pub body: String,
    /// 消息渠道类型
    pub channel: ChannelType,
}
