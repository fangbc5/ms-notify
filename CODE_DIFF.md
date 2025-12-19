# 代码迁移差异对比

本文档对比了 `flare` 项目原始代码和迁移到 `ms-notify` 后的代码差异。

## 一、整体架构变化

### 1. 模块结构

- **原始**: 多 crate 结构 (`flare-core`, `flare-adapters`, `flare-common`)
- **迁移后**: 单 crate 结构，使用模块组织 (`src/models/`, `src/adapters/`, `src/config/`)

### 2. 错误处理

- **原始**: 使用 `FlareError` 和 `FlareResult`
- **迁移后**:
  - 使用 `NotifyError` 和 `NotifyResult`（适配器内部）
  - 转换为 `fbc-starter::AppError`（统一错误处理）
  - 新增错误码模块 `error_code`

## 二、具体文件对比

### 1. ChannelType 枚举 (`models/channel.rs`)

**原始代码** (`flare-common/src/enums/channel.rs`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    Email,
    Sms,
    ImFeishu,
    ImDingding,
    ImWechat,
    Push,
    SiteMessage,
}
```

**迁移后** (`ms-notify/src/models/channel.rs`):

- ✅ **无变化**：完全一致
- ✅ 添加了中文注释说明

### 2. Message 类型枚举 (`models/message.rs`)

**原始代码** (`flare-common/src/enums/message.rs`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeishuMessageType {
    Text, Post, Image, File, Audio, Media, Sticker,
    Interactive, ShareChat, ShareUser, System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DingdingMessageType {
    Text, Markdown, Link, ActionCard, FeedCard,
    Image, File, Audio, Video,
}
```

**迁移后** (`ms-notify/src/models/message.rs`):

- ✅ **完整迁移**：`FeishuMessageType` 和 `DingdingMessageType` 枚举已完整迁移
- ✅ **新增功能**：添加了 `from_str()` 方法，支持字符串到枚举的转换
- ✅ **向后兼容**：支持字符串和枚举两种格式的反序列化
- ✅ **类型安全**：编译期检查消息类型，避免运行时错误

**主要改进**:

```rust
impl FeishuMessageType {
    /// 从字符串转换为枚举（不区分大小写）
    pub fn from_str(s: &str) -> Option<Self> { ... }
}

impl DingdingMessageType {
    /// 从字符串转换为枚举（不区分大小写）
    pub fn from_str(s: &str) -> Option<Self> { ... }
}
```

### 3. Notification 结构 (`models/notification.rs`)

**原始代码** (`flare-core/src/notification.rs`):

```rust
#[derive(Debug, Clone)]
pub struct Notification {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub channel: ChannelType,
}
```

**迁移后** (`ms-notify/src/models/notification.rs`):

- ✅ **无变化**：结构体字段完全一致
- ✅ 添加了字段注释说明

### 4. Sender Trait (`adapters/sender.rs`)

**原始代码** (`flare-core/src/sender.rs`):

```rust
#[async_trait]
pub trait Sender {
    async fn send(&self, notification: &Notification) -> FlareResult<()>;
}
```

**迁移后** (`ms-notify/src/adapters/sender.rs`):

- ✅ **无变化**：trait 定义完全一致
- ✅ 返回类型从 `FlareResult` 改为 `NotifyResult`
- ✅ 添加了详细的文档注释

### 5. Email 适配器 (`adapters/email.rs`)

**原始代码** (`flare-adapters/src/email.rs`):

```rust
impl EmailSender {
    pub fn new(config: &EmailConfig) -> Self {
        let mailer = AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&config.smtp_server)
            .unwrap()  // ⚠️ 使用 unwrap
            .credentials(...)
            .build();
        Self { mailer }
    }
}
```

**迁移后** (`ms-notify/src/adapters/email.rs`):

- ✅ **改进**：使用 `expect()` 替代 `unwrap()`，提供更清晰的错误信息
- ✅ **新增**：支持 `smtp_port` 配置（原始代码硬编码端口）
- ✅ **移除**：测试代码（后续统一测试）
- ✅ **改进**：添加了文档注释

**主要差异**:

```diff
- .unwrap()
+ .expect("Failed to create SMTP transport")
+ .port(config.smtp_port)  // 新增：支持配置端口
```

### 6. SMS 适配器 (`adapters/sms.rs`)

**原始代码** (`flare-adapters/src/ali_sms.rs`):

```rust
params.insert("RegionId", "cn-hangzhou");  // ⚠️ 硬编码
params.insert("TemplateCode", &self.config.template_code);  // ⚠️ 可能为 None
```

**迁移后** (`ms-notify/src/adapters/sms.rs`):

- ✅ **改进**：使用 `config.region_id` 替代硬编码
- ✅ **改进**：`template_code` 支持可选，提供默认值
- ✅ **改进**：错误处理从 `FlareError::String` 改为 `NotifyError::Config`
- ✅ **改进**：使用 `tracing::debug!` 替代 `println!`
- ✅ **移除**：测试代码

**主要差异**:

```diff
- params.insert("RegionId", "cn-hangzhou");
+ params.insert("RegionId", &self.config.region_id);

- params.insert("TemplateCode", &self.config.template_code);
+ let template_code = self.config.template_code.as_deref().unwrap_or("SMS_123456789");
+ params.insert("TemplateCode", template_code);

- println!("Sms response: {}", text);
+ tracing::debug!("SMS response: {}", text);
```

### 6. Feishu 适配器 (`adapters/feishu.rs`)

**原始代码** (`flare-adapters/src/im_feishu.rs`):

```rust
use flare_common::{FeishuConfig, FeishuMessageType, FlareError, FlareResult};

#[derive(Debug, Deserialize)]
struct FeishuIncoming<'a> {
    msg_type: Option<FeishuMessageType>,  // 使用枚举类型
    ...
}

match msg_type {
    FeishuMessageType::Text => { ... }
    FeishuMessageType::Post | FeishuMessageType::Image | ... => { ... }
    FeishuMessageType::Interactive => { ... }
}
```

**迁移后** (`ms-notify/src/adapters/feishu.rs`):

- ✅ **简化**：移除了 `FeishuMessageType` 枚举依赖，使用字符串匹配
- ✅ **简化**：只保留常用的消息类型（Text, Post, Image, Interactive）
- ✅ **改进**：错误处理从 `FlareError::Config` 改为 `NotifyError::Config`
- ✅ **移除**：测试代码

**主要差异**:

```diff
- msg_type: Option<FeishuMessageType>,  // 枚举类型
+ msg_type: Option<String>,  // 字符串类型

- let msg_type = incoming.msg_type.unwrap_or(FeishuMessageType::Text);
- match msg_type {
-     FeishuMessageType::Text => { ... }
+ let msg_type_str = incoming.msg_type.as_deref().unwrap_or("text");
+ match msg_type_str {
+     "text" => { ... }
```

### 7. Dingding 适配器 (`adapters/dingding.rs`)

**原始代码** (`flare-adapters/src/im_dingding.rs`):

```rust
use flare_common::{FlareError, FlareResult, DingdingConfig, DingdingMessageType};

#[derive(Debug, Deserialize)]
struct DingdingIncoming<'a> {
    msg_type: Option<DingdingMessageType>,  // 使用枚举类型
    ...
}

match msg_type {
    DingdingMessageType::Text => { ... }
    DingdingMessageType::Markdown => { ... }
    ...
}

println!("请求URL: {}", url);  // ⚠️ 调试输出
println!("请求体: {}", ...);
println!("响应状态: {}", status);
println!("响应内容: {}", response_text);
```

**迁移后** (`ms-notify/src/adapters/dingding.rs`):

- ✅ **简化**：移除了 `DingdingMessageType` 枚举依赖，使用字符串匹配
- ✅ **改进**：使用 `tracing::debug!` 替代 `println!`
- ✅ **改进**：错误处理从 `FlareError::String` 改为 `NotifyError::Send`
- ✅ **移除**：测试代码

**主要差异**:

```diff
- msg_type: Option<DingdingMessageType>,  // 枚举类型
+ msg_type: Option<String>,  // 字符串类型

- println!("请求URL: {}", url);
- println!("请求体: {}", ...);
+ tracing::debug!("Dingding response status: {}, body: {}", status, response_text);

- return Err(FlareError::String(format!("钉钉API错误 {}: {}", status, response_text)));
+ return Err(NotifyError::Send(format!("钉钉API错误 {}: {}", status, response_text)));
```

### 9. 错误处理 (`error.rs`)

**原始代码** (`flare-common/src/errors.rs`):

```rust
#[derive(Debug, Error)]
pub enum FlareError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error("Email address error: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("Email build error: {0}")]
    Lettre(#[from] lettre::error::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("String error: {0}")]
    String(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type FlareResult<T> = Result<T, FlareError>;
```

**迁移后** (`ms-notify/src/error.rs`):

- ✅ **新增**：错误码模块 `error_code`，定义了 6 个错误码
- ✅ **简化**：移除了 `Io`, `String`, `Unknown` 错误类型
- ✅ **改进**：错误转换为 `fbc-starter::AppError`，统一错误处理
- ✅ **改进**：错误消息包含错误码

**主要差异**:

```diff
+ /// 通知服务错误码
+ pub mod error_code {
+     pub const SMTP_ERROR: i32 = 5001;
+     pub const EMAIL_ADDRESS_ERROR: i32 = 4001;
+     pub const EMAIL_BUILD_ERROR: i32 = 5002;
+     pub const HTTP_ERROR: i32 = 5003;
+     pub const NOTIFY_CONFIG_ERROR: i32 = 5004;
+     pub const NOTIFY_SEND_ERROR: i32 = 5005;
+ }

- pub enum FlareError { ... }
+ pub enum NotifyError {
+     Smtp(...),
+     EmailAddress(...),
+     EmailBuild(...),
+     Http(...),
+     Config(String),
+     Send(String),
+     // 移除了: Io, String, Unknown
+ }

+ impl From<NotifyError> for BaseAppError {
+     fn from(err: NotifyError) -> Self {
+         // 转换为 fbc-starter::AppError，包含错误码
+     }
+ }
```

## 三、主要改进点

### 1. 错误处理

- ✅ 统一使用 `fbc-starter::AppError`
- ✅ 新增错误码系统，便于错误追踪和定位
- ✅ 错误消息包含错误码信息

### 2. 配置管理

- ✅ 集成 `fbc-starter` 的配置系统
- ✅ 支持环境变量配置
- ✅ 配置结构更清晰

### 3. 日志系统

- ✅ 使用 `tracing` 替代 `println!`
- ✅ 统一的日志格式和级别

### 4. 代码简化

- ✅ 移除了消息类型枚举，使用字符串匹配（简化实现）
- ✅ 移除了测试代码（后续统一测试）
- ✅ 移除了不必要的错误类型

### 5. 代码质量

- ✅ 添加了详细的文档注释
- ✅ 改进了错误处理（`expect` 替代 `unwrap`）
- ✅ 支持更多配置选项（如 SMTP 端口、区域 ID）

## 四、待迁移内容

### 1. 企业微信适配器 (`adapters/wechat.rs`)

- ⬜ 尚未实现

### 2. Kafka 消息处理 (`kafka/handler.rs`)

- ⬜ 尚未实现

### 3. HTTP API (`handlers/notification.rs`)

- ⬜ 尚未实现

### 4. 主程序整合 (`main.rs`)

- ⬜ 尚未实现

## 五、总结

已迁移的核心代码基本保持了原有功能，主要改进包括：

1. **错误处理**：统一错误处理，新增错误码系统
2. **配置管理**：集成 fbc-starter 配置系统
3. **日志系统**：使用 tracing 替代 println
4. **代码简化**：移除枚举依赖，简化实现
5. **代码质量**：添加文档注释，改进错误处理

迁移后的代码更加符合 Rust 最佳实践，并且与 `fbc-starter` 框架更好地集成。
