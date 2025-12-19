use crate::adapters::Sender;
use crate::config::EmailConfig;
use crate::error::NotifyResult;
use crate::models::Notification;
use async_trait::async_trait;
use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::AsyncTransport;

/// 邮件发送适配器
pub struct EmailSender {
    mailer: AsyncSmtpTransport<lettre::Tokio1Executor>,
}

impl EmailSender {
    /// 创建邮件发送器
    ///
    /// # 参数
    /// - `config`: 邮件配置
    pub fn new(config: &EmailConfig) -> Self {
        let mailer = AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&config.smtp_server)
            .expect("Failed to create SMTP transport")
            .port(config.smtp_port)
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                config.smtp_user.clone(),
                config.smtp_pass.clone(),
            ))
            .build();

        Self { mailer }
    }
}

#[async_trait]
impl Sender for EmailSender {
    async fn send(&self, notification: &Notification) -> NotifyResult<()> {
        let email = Message::builder()
            .from(notification.from.parse::<Mailbox>()?)
            .to(notification.to.parse::<Mailbox>()?)
            .subject(&notification.subject)
            .body(notification.body.clone())?;

        self.mailer.send(email).await?;
        Ok(())
    }
}
