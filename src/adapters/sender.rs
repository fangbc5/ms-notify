use crate::error::NotifyResult;
use crate::models::Notification;
use async_trait::async_trait;

/// 消息发送器 trait
/// 所有消息适配器都需要实现此 trait
#[async_trait]
pub trait Sender {
    /// 发送通知消息
    ///
    /// # 参数
    /// - `notification`: 通知消息
    ///
    /// # 返回
    /// - `Ok(())`: 发送成功
    /// - `Err(NotifyError)`: 发送失败
    async fn send(&self, notification: &Notification) -> NotifyResult<()>;
}
