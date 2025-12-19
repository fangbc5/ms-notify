use axum::response::Json;
use fbc_starter::R;
use serde::Serialize;

/// 渠道信息
#[derive(Debug, Serialize)]
pub struct ChannelInfo {
    /// 渠道类型
    pub channel: String,
    /// 渠道名称
    pub name: String,
    /// 是否支持
    pub supported: bool,
}

/// 获取支持的渠道列表处理器
pub async fn list_channels() -> Json<R<Vec<ChannelInfo>>> {
    let channels = vec![
        ChannelInfo {
            channel: "email".to_string(),
            name: "邮件".to_string(),
            supported: true,
        },
        ChannelInfo {
            channel: "sms".to_string(),
            name: "短信".to_string(),
            supported: true,
        },
        ChannelInfo {
            channel: "im_feishu".to_string(),
            name: "飞书".to_string(),
            supported: true,
        },
        ChannelInfo {
            channel: "im_dingding".to_string(),
            name: "钉钉".to_string(),
            supported: true,
        },
        ChannelInfo {
            channel: "im_wechat".to_string(),
            name: "企业微信".to_string(),
            supported: false,
        },
        ChannelInfo {
            channel: "push".to_string(),
            name: "推送通知".to_string(),
            supported: false,
        },
        ChannelInfo {
            channel: "site_message".to_string(),
            name: "站内消息".to_string(),
            supported: false,
        },
    ];

    Json(R::ok_with_data(channels))
}
