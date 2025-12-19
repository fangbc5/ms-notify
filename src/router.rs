use crate::handlers::{list_channels, send_notification};
use crate::kafka::NotificationHandlerContext;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// 创建 HTTP 路由
///
/// # 参数
/// - `context`: 通知处理器上下文，包含所有消息发送器
///
/// 注意：健康检查路由 `/health` 已由 fbc-starter 在基础路由中提供
pub fn create_router(context: Arc<NotificationHandlerContext>) -> Router {
    Router::new()
        // API v1 路由
        .nest(
            "/api/v1",
            Router::new()
                .route("/notifications", post(send_notification))
                .route("/channels", get(list_channels))
                .with_state(context),
        )
}
