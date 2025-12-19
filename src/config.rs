use fbc_starter::Config as BaseConfig;
use serde::{Deserialize, Serialize};

/// 通知服务配置
/// 扩展 fbc-starter 的配置，添加通知相关的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyConfig {
    /// 基础配置（继承自 fbc-starter）
    #[serde(flatten)]
    pub base: BaseConfig,
    /// 通知配置
    pub notify: NotifyServiceConfig,
}

/// 通知服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyServiceConfig {
    /// 邮件配置
    #[serde(default)]
    pub email: Option<EmailConfig>,
    /// 短信配置
    #[serde(default)]
    pub sms: Option<SmsConfig>,
    /// 飞书配置
    #[serde(default)]
    pub feishu: Option<FeishuConfig>,
    /// 钉钉配置
    #[serde(default)]
    pub dingding: Option<DingdingConfig>,
    /// 企业微信配置（可选）
    #[serde(default)]
    pub wechat: Option<WechatConfig>,
}

/// 邮件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP 服务器地址
    pub smtp_server: String,
    /// SMTP 用户名
    pub smtp_user: String,
    /// SMTP 密码
    pub smtp_pass: String,
    /// SMTP 端口（可选，默认 587）
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
}

fn default_smtp_port() -> u16 {
    587
}

/// 短信配置（阿里云）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    /// 短信服务端点
    pub endpoint: String,
    /// Access Key ID
    pub access_key_id: String,
    /// Access Key Secret
    pub access_key_secret: String,
    /// 签名名称
    pub sign_name: String,
    /// 模板代码（默认模板）
    #[serde(default)]
    pub template_code: Option<String>,
    /// 区域 ID（可选，默认 cn-hangzhou）
    #[serde(default = "default_region_id")]
    pub region_id: String,
}

fn default_region_id() -> String {
    "cn-hangzhou".to_string()
}

/// 飞书配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuConfig {
    /// Webhook URL
    pub webhook: String,
    /// 签名密钥（可选）
    #[serde(default)]
    pub secret: Option<String>,
}

/// 钉钉配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DingdingConfig {
    /// Webhook URL
    pub webhook: String,
    /// 签名密钥（可选）
    #[serde(default)]
    pub secret: Option<String>,
}

/// 企业微信配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WechatConfig {
    /// Webhook URL
    pub webhook: String,
}

impl NotifyConfig {
    /// 从环境变量加载配置
    /// 使用 fbc-starter 的配置加载机制
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // 加载基础配置
        let base_config = BaseConfig::from_env()?;

        // 加载通知相关配置
        // 使用 APP__NOTIFY__ 前缀的环境变量
        let notify_config = config::Config::builder()
            .add_source(config::Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize::<NotifyServiceConfig>()?;

        Ok(Self {
            base: base_config,
            notify: notify_config,
        })
    }
}
