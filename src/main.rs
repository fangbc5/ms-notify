use crate::config::NotifyConfig;
use crate::kafka::{NotificationHandler, NotificationHandlerContext};
use fbc_starter::{AppResult, Server};
use std::sync::Arc;

mod adapters;
mod config;
mod error;
mod handlers;
mod kafka;
mod models;
mod router;

#[tokio::main]
async fn main() -> AppResult<()> {
    // 加载配置
    let config = NotifyConfig::from_env()?;

    // 创建 Kafka 处理器上下文
    let context = Arc::new(NotificationHandlerContext::new(&config));

    // 创建 HTTP 路由（需要在创建 handler 之前克隆 context）
    let http_router = router::create_router(context.clone());

    // 创建 Kafka 消息处理器
    // topic 和 group_id 会由框架自动从 handler.topics() 和 handler.group_id() 获取
    let handler: Arc<dyn fbc_starter::KafkaMessageHandler> =
        Arc::new(NotificationHandler::new(context));

    // 启动服务器，注册 Kafka 处理器和 HTTP 路由
    Server::run(|builder| builder.with_kafka_handler(handler).http_router(http_router)).await
}
