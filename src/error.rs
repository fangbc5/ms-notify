// 通知服务错误类型
// 扩展 fbc-starter 的 AppError，添加通知相关的错误转换

use fbc_starter::AppError as BaseAppError;

/// 通知服务错误码
pub mod error_code {
    /// SMTP 错误
    pub const SMTP_ERROR: i32 = 5001;
    /// 邮件地址错误
    pub const EMAIL_ADDRESS_ERROR: i32 = 4001;
    /// 邮件构建错误
    pub const EMAIL_BUILD_ERROR: i32 = 5002;
    /// HTTP 请求错误
    pub const HTTP_ERROR: i32 = 5003;
    /// 通知配置错误
    pub const NOTIFY_CONFIG_ERROR: i32 = 5004;
    /// 通知发送失败
    pub const NOTIFY_SEND_ERROR: i32 = 5005;
}

/// 通知服务错误类型
/// 用于适配器内部错误处理，最终转换为 AppError
#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    /// SMTP 错误
    #[error("SMTP 错误: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),

    /// 邮件地址错误
    #[error("邮件地址错误: {0}")]
    EmailAddress(#[from] lettre::address::AddressError),

    /// 邮件构建错误
    #[error("邮件构建错误: {0}")]
    EmailBuild(#[from] lettre::error::Error),

    /// HTTP 请求错误
    #[error("HTTP 请求错误: {0}")]
    Http(#[from] reqwest::Error),

    /// 通知配置错误
    #[error("通知配置错误: {0}")]
    Config(String),

    /// 通知发送失败
    #[error("通知发送失败: {0}")]
    Send(String),
}

/// 将 NotifyError 转换为 AppError
impl From<NotifyError> for BaseAppError {
    fn from(err: NotifyError) -> Self {
        use error_code::*;
        match err {
            NotifyError::Smtp(e) => {
                BaseAppError::BizError(SMTP_ERROR as i32, format!("SMTP 错误: {}", e))
            }
            NotifyError::EmailAddress(e) => {
                BaseAppError::BizError(EMAIL_ADDRESS_ERROR as i32, format!("邮件地址错误: {}", e))
            }
            NotifyError::EmailBuild(e) => {
                BaseAppError::BizError(EMAIL_BUILD_ERROR as i32, format!("邮件构建错误: {}", e))
            }
            NotifyError::Http(e) => {
                BaseAppError::BizError(HTTP_ERROR, format!("HTTP 请求错误: {}", e))
            }
            NotifyError::Config(msg) => {
                BaseAppError::BizError(NOTIFY_CONFIG_ERROR, format!("通知配置错误: {}", msg))
            }
            NotifyError::Send(msg) => {
                BaseAppError::BizError(NOTIFY_SEND_ERROR, format!("通知发送失败: {}", msg))
            }
        }
    }
}

/// 通知服务结果类型
pub type NotifyResult<T> = Result<T, NotifyError>;
