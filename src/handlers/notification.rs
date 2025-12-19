use crate::kafka::NotificationHandlerContext;
use crate::models::{ChannelType, Notification};
use axum::{extract::State, response::Json};
use fbc_starter::{AppResult, R};
use serde::Deserialize;
use std::sync::Arc;

/// 发送通知请求
#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    /// 发送者（邮件时使用，可选）
    #[serde(default)]
    pub from: String,
    /// 接收者（邮件、短信时使用）
    pub to: String,
    /// 主题（邮件时使用，可选）
    #[serde(default)]
    pub subject: String,
    /// 消息内容
    pub body: String,
    /// 消息渠道类型
    pub channel: ChannelType,
}

/// 发送通知处理器
pub async fn send_notification(
    State(context): State<Arc<NotificationHandlerContext>>,
    Json(request): Json<SendNotificationRequest>,
) -> AppResult<Json<R<String>>> {
    // 构建通知消息
    let notification = Notification {
        from: request.from,
        to: request.to,
        subject: request.subject,
        body: request.body,
        channel: request.channel,
    };

    // 使用上下文的方法发送通知
    context.send(&notification).await?;

    Ok(Json(R::ok_with_data(
        "Notification sent successfully".to_string(),
    )))
}
