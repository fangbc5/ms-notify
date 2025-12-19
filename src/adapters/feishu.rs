use crate::adapters::Sender;
use crate::config::FeishuConfig;
use crate::error::{NotifyError, NotifyResult};
use crate::models::{FeishuMessageType, Notification};
use async_trait::async_trait;
use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;

/// 飞书发送器
pub struct FeishuSender {
    client: Client,
    config: FeishuConfig,
}

impl FeishuSender {
    /// 创建飞书发送器
    ///
    /// # 参数
    /// - `config`: 飞书配置
    pub fn new(config: FeishuConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

#[derive(Debug, Deserialize)]
struct FeishuIncoming<'a> {
    /// 消息类型（支持枚举或字符串）
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_msg_type")]
    msg_type: Option<FeishuMessageType>,
    #[serde(default)]
    content: Option<serde_json::Value>,
    #[serde(default)]
    card: Option<serde_json::Value>,
    // 兼容直接传 {"text":"..."}
    #[serde(default)]
    text: Option<&'a str>,
}

/// 反序列化消息类型：支持枚举和字符串两种格式
fn deserialize_msg_type<'de, D>(deserializer: D) -> Result<Option<FeishuMessageType>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    // 先尝试作为字符串反序列化
    let value: Option<String> = Option::deserialize(deserializer)?;
    match value {
        Some(s) => FeishuMessageType::from_str(&s)
            .ok_or_else(|| D::Error::custom(format!("Invalid Feishu message type: {}", s)))
            .map(Some),
        None => Ok(None),
    }
}

#[async_trait]
impl Sender for FeishuSender {
    async fn send(&self, notification: &Notification) -> NotifyResult<()> {
        let mut url = self.config.webhook.clone();

        // 如果配置了 secret，需要附带签名
        if let Some(secret) = &self.config.secret {
            let ts = chrono::Utc::now().timestamp();
            let string_to_sign = format!("{}\n{}", ts, secret);

            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
                .map_err(|e| NotifyError::Config(format!("feishu secret error: {}", e)))?;
            mac.update(string_to_sign.as_bytes());
            let sign =
                base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

            let sep = if url.contains('?') { '&' } else { '?' };
            url = format!("{}{}timestamp={}&sign=\"{}\"", url, sep, ts, sign);
        }

        // 解析 notification.body 来支持多类型
        // 支持两种格式：
        // 1. JSON 对象格式：{"msg_type": "text", "content": {...}}
        // 2. 纯文本格式：直接作为文本消息发送
        let parsed: Result<FeishuIncoming, _> = serde_json::from_str(&notification.body);
        let body_value = match parsed {
            Ok(incoming) => {
                let msg_type = incoming.msg_type.unwrap_or(FeishuMessageType::Text);
                match msg_type {
                    FeishuMessageType::Text => {
                        let text = incoming
                            .content
                            .as_ref()
                            .and_then(|v| v.get("text").and_then(|x| x.as_str()))
                            .or(incoming.text)
                            .unwrap_or(notification.body.as_str());
                        json!({
                            "msg_type": "text",
                            "content": { "text": text }
                        })
                    }
                    FeishuMessageType::Post => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "post",
                            "content": content
                        })
                    }
                    FeishuMessageType::Image => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "image",
                            "content": content
                        })
                    }
                    FeishuMessageType::File => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "file",
                            "content": content
                        })
                    }
                    FeishuMessageType::Audio => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "audio",
                            "content": content
                        })
                    }
                    FeishuMessageType::Media => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "media",
                            "content": content
                        })
                    }
                    FeishuMessageType::Sticker => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "sticker",
                            "content": content
                        })
                    }
                    FeishuMessageType::Interactive => {
                        let card = incoming.card.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "interactive",
                            "card": card
                        })
                    }
                    FeishuMessageType::ShareChat => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "share_chat",
                            "content": content
                        })
                    }
                    FeishuMessageType::ShareUser => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "share_user",
                            "content": content
                        })
                    }
                    FeishuMessageType::System => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({
                            "msg_type": "system",
                            "content": content
                        })
                    }
                }
            }
            Err(_) => {
                // 如果不是 JSON 格式，作为纯文本消息发送
                json!({ "msg_type": "text", "content": { "text": notification.body } })
            }
        };

        self.client
            .post(&url)
            .json(&body_value)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
