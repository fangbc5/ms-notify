use crate::adapters::Sender;
use crate::config::DingdingConfig;
use crate::error::{NotifyError, NotifyResult};
use crate::models::{DingdingMessageType, Notification};
use async_trait::async_trait;
use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;

/// 钉钉发送器
pub struct DingdingSender {
    client: Client,
    config: DingdingConfig,
}

impl DingdingSender {
    /// 创建钉钉发送器
    ///
    /// # 参数
    /// - `config`: 钉钉配置
    pub fn new(config: DingdingConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

#[derive(Debug, Deserialize)]
struct DingdingIncoming<'a> {
    /// 消息类型（支持枚举或字符串）
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_msg_type")]
    msg_type: Option<DingdingMessageType>,
    #[serde(default)]
    content: Option<serde_json::Value>,
    #[serde(default)]
    text: Option<&'a str>,
}

/// 反序列化消息类型：支持枚举和字符串两种格式
fn deserialize_msg_type<'de, D>(deserializer: D) -> Result<Option<DingdingMessageType>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value: Option<String> = Option::deserialize(deserializer)?;
    match value {
        Some(s) => DingdingMessageType::from_str(&s)
            .ok_or_else(|| D::Error::custom(format!("Invalid Dingding message type: {}", s)))
            .map(Some),
        None => Ok(None),
    }
}

#[async_trait]
impl Sender for DingdingSender {
    async fn send(&self, notification: &Notification) -> NotifyResult<()> {
        let mut url = self.config.webhook.clone();

        if let Some(secret) = &self.config.secret {
            let ts = chrono::Utc::now().timestamp_millis();
            let string_to_sign = format!("{}\n{}", ts, secret);
            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
                .map_err(|e| NotifyError::Config(format!("dingding secret error: {}", e)))?;
            mac.update(string_to_sign.as_bytes());
            let sign =
                base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
            let sep = if url.contains('?') { '&' } else { '?' };
            url = format!(
                "{}{}timestamp={}&sign={}",
                url,
                sep,
                ts,
                urlencoding::encode(&sign)
            );
        }

        // 解析 body：允许直接传 text，或 content 对象
        // 支持两种格式：
        // 1. JSON 对象格式：{"msg_type": "text", "content": {...}}
        // 2. 纯文本格式：直接作为文本消息发送
        let parsed: Result<DingdingIncoming, _> = serde_json::from_str(&notification.body);
        let body_value = match parsed {
            Ok(incoming) => {
                let msg_type = incoming.msg_type.unwrap_or(DingdingMessageType::Text);
                match msg_type {
                    DingdingMessageType::Text => {
                        let text = incoming
                            .content
                            .as_ref()
                            .and_then(|v| v.get("content").and_then(|x| x.as_str()))
                            .or(incoming.text)
                            .unwrap_or(notification.body.as_str());
                        json!({ "msgtype": "text", "text": { "content": text } })
                    }
                    DingdingMessageType::Markdown => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "markdown", "markdown": content })
                    }
                    DingdingMessageType::Link => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "link", "link": content })
                    }
                    DingdingMessageType::ActionCard => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "actionCard", "actionCard": content })
                    }
                    DingdingMessageType::FeedCard => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "feedCard", "feedCard": content })
                    }
                    DingdingMessageType::Image => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "image", "image": content })
                    }
                    DingdingMessageType::File => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "file", "file": content })
                    }
                    DingdingMessageType::Audio => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "audio", "audio": content })
                    }
                    DingdingMessageType::Video => {
                        let content = incoming.content.unwrap_or_else(|| json!({}));
                        json!({ "msgtype": "video", "video": content })
                    }
                }
            }
            Err(_) => {
                // 如果不是 JSON 格式，作为纯文本消息发送
                json!({ "msgtype": "text", "text": { "content": notification.body } })
            }
        };

        let response = self.client.post(&url).json(&body_value).send().await?;

        let status = response.status();
        let response_text = response.text().await?;

        tracing::debug!(
            "Dingding response status: {}, body: {}",
            status,
            response_text
        );

        if !status.is_success() {
            return Err(NotifyError::Send(format!(
                "钉钉API错误 {}: {}",
                status, response_text
            )));
        }

        Ok(())
    }
}
