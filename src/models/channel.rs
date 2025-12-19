use serde::{Deserialize, Serialize};

/// 消息渠道类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// 邮件
    Email,
    /// 短信
    Sms,
    /// 飞书
    ImFeishu,
    /// 钉钉
    ImDingding,
    /// 企业微信
    ImWechat,
    /// 推送通知
    Push,
    /// 站内消息
    SiteMessage,
}
