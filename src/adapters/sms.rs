use crate::adapters::Sender;
use crate::config::SmsConfig;
use crate::error::{NotifyError, NotifyResult};
use crate::models::Notification;
use async_trait::async_trait;
use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Serialize;
use sha1::Sha1;
use std::collections::HashMap;
use uuid::Uuid;

/// 阿里云短信API请求参数
#[derive(Serialize)]
struct SmsRequest<'a> {
    #[serde(rename = "PhoneNumbers")]
    phone_numbers: &'a str,
    #[serde(rename = "SignName")]
    sign_name: &'a str,
    #[serde(rename = "TemplateCode")]
    template_code: &'a str,
    #[serde(rename = "TemplateParam")]
    template_param: &'a str,
    #[serde(rename = "Action")]
    action: &'a str,
    #[serde(rename = "Version")]
    version: &'a str,
    #[serde(rename = "RegionId")]
    region_id: &'a str,
    #[serde(rename = "AccessKeyId")]
    access_key_id: &'a str,
    #[serde(rename = "Signature")]
    signature: String,
    #[serde(rename = "SignatureMethod")]
    signature_method: &'a str,
    #[serde(rename = "SignatureVersion")]
    signature_version: &'a str,
    #[serde(rename = "SignatureNonce")]
    signature_nonce: String,
    #[serde(rename = "Timestamp")]
    timestamp: String,
}

/// 短信发送器
pub struct SmsSender {
    client: Client,
    config: SmsConfig,
}

impl SmsSender {
    /// 创建短信发送器
    ///
    /// # 参数
    /// - `config`: 短信配置
    pub fn new(config: SmsConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    async fn send_sms(&self, phone: &str, param_json: &str) -> NotifyResult<()> {
        // 生成时间戳和随机数 (ISO 8601 格式)
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = Uuid::new_v4().to_string();

        // 构建请求参数
        let mut params = HashMap::new();
        params.insert("Action", "SendSms");
        params.insert("Version", "2017-05-25");
        params.insert("RegionId", &self.config.region_id);
        params.insert("PhoneNumbers", phone);
        params.insert("SignName", &self.config.sign_name);
        let template_code = self
            .config
            .template_code
            .as_deref()
            .unwrap_or("SMS_123456789");
        params.insert("TemplateCode", template_code);
        params.insert("TemplateParam", param_json);
        params.insert("AccessKeyId", &self.config.access_key_id);
        params.insert("SignatureMethod", "HMAC-SHA1");
        params.insert("SignatureVersion", "1.0");
        params.insert("SignatureNonce", &nonce);
        params.insert("Timestamp", &timestamp);

        // 生成签名
        let mut sorted_params: Vec<_> = params.iter().collect();
        sorted_params.sort_by_key(|(k, _)| *k);

        let query_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let string_to_sign = format!(
            "POST&{}&{}",
            urlencoding::encode("/"),
            urlencoding::encode(&query_string)
        );

        let signing_key = format!("{}&", self.config.access_key_secret);
        let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes())
            .map_err(|e| NotifyError::Config(format!("HMAC key error: {}", e)))?;
        mac.update(string_to_sign.as_bytes());
        let signature =
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        // 构建最终请求
        let req = SmsRequest {
            phone_numbers: phone,
            sign_name: &self.config.sign_name,
            template_code,
            template_param: param_json,
            action: "SendSms",
            version: "2017-05-25",
            region_id: &self.config.region_id,
            access_key_id: &self.config.access_key_id,
            signature,
            signature_method: "HMAC-SHA1",
            signature_version: "1.0",
            signature_nonce: nonce,
            timestamp,
        };

        let resp = self
            .client
            .post(&self.config.endpoint)
            .form(&req)
            .send()
            .await?;

        let text = resp.text().await?;
        tracing::debug!("SMS response: {}", text);

        // 可在这里根据返回判断是否发送成功
        Ok(())
    }
}

#[async_trait]
impl Sender for SmsSender {
    async fn send(&self, notification: &Notification) -> NotifyResult<()> {
        // 假设 Notification.to 是手机号，body 是 JSON 参数字符串
        self.send_sms(&notification.to, &notification.body).await
    }
}
