use crate::adapters::Sender;
use crate::adapters::{DingdingSender, EmailSender, FeishuSender, SmsSender};
use crate::config::NotifyConfig;
use crate::error::NotifyError;
use crate::models::{ChannelType, Notification};
use async_trait::async_trait;
use fbc_starter::{KafkaMessageHandler, Message as KafkaMessage};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Kafka 消息处理器上下文
/// 包含所有消息发送器
pub struct NotificationHandlerContext {
    email_sender: Option<EmailSender>,
    sms_sender: Option<SmsSender>,
    feishu_sender: Option<FeishuSender>,
    dingding_sender: Option<DingdingSender>,
    /// 邮件配置（用于获取默认发件人）
    email_config: Option<crate::config::EmailConfig>,
}

impl NotificationHandlerContext {
    /// 创建处理器上下文
    pub fn new(config: &NotifyConfig) -> Self {
        Self {
            email_sender: config
                .notify
                .email
                .as_ref()
                .map(|cfg| EmailSender::new(cfg)),
            sms_sender: config.notify.sms.clone().map(|cfg| SmsSender::new(cfg)),
            feishu_sender: config
                .notify
                .feishu
                .clone()
                .map(|cfg| FeishuSender::new(cfg)),
            dingding_sender: config
                .notify
                .dingding
                .clone()
                .map(|cfg| DingdingSender::new(cfg)),
            email_config: config.notify.email.clone(),
        }
    }

    /// 发送通知消息
    /// 供 HTTP handlers 和 Kafka handlers 使用
    pub async fn send(&self, notification: &Notification) -> Result<(), NotifyError> {
        match notification.channel {
            ChannelType::Email => {
                let sender = self.email_sender.as_ref().ok_or_else(|| {
                    NotifyError::Config("Email sender not configured".to_string())
                })?;

                // 如果 from 为空，使用配置的默认发件人
                let notification = if notification.from.is_empty() {
                    let from = self
                        .email_config
                        .as_ref()
                        .map(|cfg| cfg.smtp_user.clone())
                        .unwrap_or_else(|| "noreply@example.com".to_string());
                    Notification {
                        from,
                        ..notification.clone()
                    }
                } else {
                    notification.clone()
                };

                sender.send(&notification).await?;
            }
            ChannelType::Sms => {
                let sender = self
                    .sms_sender
                    .as_ref()
                    .ok_or_else(|| NotifyError::Config("SMS sender not configured".to_string()))?;
                sender.send(notification).await?;
            }
            ChannelType::ImFeishu => {
                let sender = self.feishu_sender.as_ref().ok_or_else(|| {
                    NotifyError::Config("Feishu sender not configured".to_string())
                })?;
                sender.send(notification).await?;
            }
            ChannelType::ImDingding => {
                let sender = self.dingding_sender.as_ref().ok_or_else(|| {
                    NotifyError::Config("Dingding sender not configured".to_string())
                })?;
                sender.send(notification).await?;
            }
            _ => {
                return Err(NotifyError::Config(format!(
                    "Unsupported channel type: {:?}",
                    notification.channel
                )));
            }
        }
        Ok(())
    }
}

/// Kafka 通知消息处理器
/// 实现 KafkaMessageHandler trait，由 fbc-starter 自动管理订阅和消息分发
pub struct NotificationHandler {
    context: Arc<NotificationHandlerContext>,
}

impl NotificationHandler {
    /// 创建新的处理器
    ///
    /// # 参数
    /// - `context`: 处理器上下文
    pub fn new(context: Arc<NotificationHandlerContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl KafkaMessageHandler for NotificationHandler {
    /// 返回此 handler 处理的 topic 列表
    /// 框架会自动调用此方法来获取需要订阅的 topics
    fn topics(&self) -> Vec<String> {
        // 从环境变量获取，或使用默认值
        vec!["flare-messages".to_string()]
    }

    /// 返回此 handler 使用的消费者组ID
    /// 框架会自动调用此方法来创建对应的消费者组
    fn group_id(&self) -> String {
        // 从环境变量获取，或使用默认值
        "flare-workers".to_string()
    }

    /// 处理接收到的消息（由 fbc-starter 自动调用）
    async fn handle(&self, message: KafkaMessage) {
        info!(
            "Received Kafka message: topic={}, from={}",
            message.topic, message.from
        );

        // 解析消息数据（message.data 是 serde_json::Value）
        // 支持两种格式：
        // 1. 直接是 Notification 格式：{from, to, subject, body, channel}
        // 2. 兼容 flare-worker 格式：{id, timestamp, source, channel, payload}
        match serde_json::from_value::<Notification>(message.data.clone()) {
            Ok(notification) => {
                // 直接使用 Notification
                if let Err(e) = dispatch(&self.context, notification).await {
                    error!("Failed to dispatch notification: {}", e);
                }
            }
            Err(_) => {
                // 尝试解析为 flare-worker 格式
                match parse_flare_format(&message.data) {
                    Ok(notification) => {
                        if let Err(e) = dispatch(&self.context, notification).await {
                            error!("Failed to dispatch notification: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to parse notification message: {}, data: {}",
                            e, message.data
                        );
                    }
                }
            }
        }
    }
}

/// 解析 flare-worker 格式的消息
fn parse_flare_format(data: &serde_json::Value) -> Result<Notification, NotifyError> {
    let channel = data
        .get("channel")
        .and_then(|v| serde_json::from_value::<ChannelType>(v.clone()).ok())
        .ok_or_else(|| NotifyError::Config("missing or invalid 'channel' field".to_string()))?;

    let payload = data
        .get("payload")
        .ok_or_else(|| NotifyError::Config("missing 'payload' field".to_string()))?;

    match channel {
        ChannelType::Email => {
            let from = payload
                .get("from")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "noreply@example.com".to_string());
            let to = require_str(payload, "to")?;
            let subject = require_str(payload, "subject")?;
            let body = require_str(payload, "body")?;
            Ok(Notification {
                from,
                to,
                subject,
                body,
                channel,
            })
        }
        ChannelType::Sms => {
            let to = require_str(payload, "to")?;
            let body = require_str(payload, "param").or_else(|_| require_str(payload, "body"))?;
            Ok(Notification {
                from: String::new(),
                to,
                subject: String::new(),
                body,
                channel,
            })
        }
        ChannelType::ImFeishu | ChannelType::ImDingding => {
            let body = require_str(payload, "text").or_else(|_| require_str(payload, "body"))?;
            Ok(Notification {
                from: String::new(),
                to: String::new(),
                subject: String::new(),
                body,
                channel,
            })
        }
        _ => Err(NotifyError::Config(format!(
            "Unsupported channel type: {:?}",
            channel
        ))),
    }
}

/// 分发消息到对应的处理器
async fn dispatch(
    ctx: &NotificationHandlerContext,
    notification: Notification,
) -> Result<(), NotifyError> {
    ctx.send(&notification).await?;
    info!(
        "Notification sent successfully: channel={:?}, to={}",
        notification.channel, notification.to
    );
    Ok(())
}

/// 从 payload 中获取字符串字段
fn require_str(payload: &serde_json::Value, key: &str) -> Result<String, NotifyError> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            NotifyError::Config(format!("missing or invalid '{}' field in payload", key))
        })
}
