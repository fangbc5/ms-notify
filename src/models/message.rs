use serde::{Deserialize, Serialize};

/// 飞书消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeishuMessageType {
    /// 文本
    Text,
    /// 富文本
    Post,
    /// 图片
    Image,
    /// 文件
    File,
    /// 语音
    Audio,
    /// 视频
    Media,
    /// 表情包
    Sticker,
    /// 卡片
    Interactive,
    /// 分享群名片
    ShareChat,
    /// 分享个人名片
    ShareUser,
    /// 系统消息
    System,
}

impl From<FeishuMessageType> for &'static str {
    fn from(t: FeishuMessageType) -> Self {
        match t {
            FeishuMessageType::Text => "text",
            FeishuMessageType::Post => "post",
            FeishuMessageType::Image => "image",
            FeishuMessageType::File => "file",
            FeishuMessageType::Audio => "audio",
            FeishuMessageType::Media => "media",
            FeishuMessageType::Sticker => "sticker",
            FeishuMessageType::Interactive => "interactive",
            FeishuMessageType::ShareChat => "share_chat",
            FeishuMessageType::ShareUser => "share_user",
            FeishuMessageType::System => "system",
        }
    }
}

impl FeishuMessageType {
    /// 从字符串转换为枚举（不区分大小写）
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" => Some(Self::Text),
            "post" => Some(Self::Post),
            "image" => Some(Self::Image),
            "file" => Some(Self::File),
            "audio" => Some(Self::Audio),
            "media" => Some(Self::Media),
            "sticker" => Some(Self::Sticker),
            "interactive" => Some(Self::Interactive),
            "share_chat" | "sharechat" => Some(Self::ShareChat),
            "share_user" | "shareuser" => Some(Self::ShareUser),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

/// 钉钉消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DingdingMessageType {
    /// 文本消息
    Text,
    /// Markdown 富文本
    Markdown,
    /// 链接消息
    Link,
    /// 交互卡片
    ActionCard,
    /// 消息卡片
    FeedCard,
    /// 图片
    Image,
    /// 文件
    File,
    /// 语音
    Audio,
    /// 视频
    Video,
}

impl From<DingdingMessageType> for &'static str {
    fn from(t: DingdingMessageType) -> Self {
        match t {
            DingdingMessageType::Text => "text",
            DingdingMessageType::Markdown => "markdown",
            DingdingMessageType::Link => "link",
            DingdingMessageType::ActionCard => "actionCard",
            DingdingMessageType::FeedCard => "feedCard",
            DingdingMessageType::Image => "image",
            DingdingMessageType::File => "file",
            DingdingMessageType::Audio => "audio",
            DingdingMessageType::Video => "video",
        }
    }
}

impl DingdingMessageType {
    /// 从字符串转换为枚举（不区分大小写）
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" => Some(Self::Text),
            "markdown" => Some(Self::Markdown),
            "link" => Some(Self::Link),
            "actioncard" | "action_card" => Some(Self::ActionCard),
            "feedcard" | "feed_card" => Some(Self::FeedCard),
            "image" => Some(Self::Image),
            "file" => Some(Self::File),
            "audio" => Some(Self::Audio),
            "video" => Some(Self::Video),
            _ => None,
        }
    }
}
