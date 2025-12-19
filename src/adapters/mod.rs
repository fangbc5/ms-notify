mod dingding;
mod email;
mod feishu;
mod sender;
mod sms;
mod wechat;

// 导出 Sender trait
pub use sender::Sender;

// 导出适配器
pub use dingding::DingdingSender;
pub use email::EmailSender;
pub use feishu::FeishuSender;
pub use sms::SmsSender;
