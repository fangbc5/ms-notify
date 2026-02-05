# Flare 项目迁移到 ms-notify 详细计划

## 一、项目分析

### Flare 项目当前架构

```
flare/
├── flare-api          # API 层（基本为空，可移除）
├── flare-core         # 核心：Notification, Sender trait
├── flare-adapters     # 适配器：Email, SMS, Feishu, Dingding, WeChat
├── flare-storage      # 存储层（基本为空，可移除）
├── flare-worker       # Kafka 消费者（Worker）
└── flare-common       # 公共：配置、枚举、错误
```

### 核心功能

1. **多渠道消息发送**：邮件、短信、飞书、钉钉、企业微信
2. **异步处理**：基于 Kafka 的消息队列
3. **统一接口**：`Sender` trait 抽象
4. **配置管理**：环境变量配置

### 简化目标

- ✅ 整合到 ms-notify 单一项目
- ✅ 使用 fbc-starter 基础设施（Kafka、Redis、MySQL）
- ✅ 添加 HTTP API（Axum）
- ✅ 简化配置管理
- ✅ 保留核心功能

---

## 二、迁移计划

### 阶段 1：项目结构设计（1-2 天）

**目标**：设计 ms-notify 的模块结构

```
ms-notify/
├── src/
│   ├── main.rs                    # 入口：HTTP Server + Kafka Consumer
│   ├── config.rs                  # 配置管理（使用 fbc-starter）
│   ├── error.rs                   # 错误类型（整合 FlareError）
│   ├── handlers/                  # HTTP 处理器
│   │   ├── mod.rs
│   │   ├── notification.rs        # 发送通知 API
│   │   └── health.rs              # 健康检查
│   ├── kafka/                     # Kafka 消费者
│   │   ├── mod.rs
│   │   ├── consumer.rs            # 消费者逻辑
│   │   └── handler.rs             # 消息处理分发
│   ├── adapters/                  # 消息适配器（从 flare-adapters 迁移）
│   │   ├── mod.rs
│   │   ├── sender.rs              # Sender trait（从 flare-core）
│   │   ├── email.rs               # 邮件发送
│   │   ├── sms.rs                 # 短信发送（阿里云）
│   │   ├── feishu.rs              # 飞书机器人
│   │   ├── dingding.rs            # 钉钉机器人
│   │   └── wechat.rs              # 企业微信（可选）
│   ├── models/                    # 数据模型
│   │   ├── mod.rs
│   │   ├── notification.rs        # Notification 结构（从 flare-core）
│   │   └── message.rs             # Kafka 消息格式
│   └── router.rs                  # Axum 路由定义
└── Cargo.toml
```

**任务清单**：

- [x] 创建目录结构
- [x] 创建各模块的 `mod.rs` 文件
- [x] 定义模块导出

---

### 阶段 2：依赖和配置（1 天）

**目标**：更新 Cargo.toml，添加必要依赖

**需要添加的依赖**：

```toml
[dependencies]
# 已有
fbc-starter = { path = "../fbc-starter", features = ["redis", "nacos", "kafka", "consumer"] }
tokio.workspace = true
async-trait.workspace = true
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true

# 新增
# 邮件发送
lettre = { version = "0.11", default-features = false, features = ["tokio1-rustls", "builder", "ring", "rustls-native-certs", "smtp-transport"] }

# HTTP 客户端（已有 reqwest）
reqwest.workspace = true

# 加密签名（飞书、钉钉）
hmac = "0.12"
sha2 = "0.10"
base64.workspace = true

# UUID（已有）
uuid.workspace = true

# 错误处理（已有）
thiserror.workspace = true
anyhow.workspace = true
```

**配置结构**（使用 fbc-starter 的配置系统）：

```rust
// src/config.rs
pub struct NotifyConfig {
    // 邮件配置
    pub email: EmailConfig,
    // 短信配置
    pub sms: SmsConfig,
    // 飞书配置
    pub feishu: FeishuConfig,
    // 钉钉配置
    pub dingding: DingdingConfig,
    // Kafka 配置（使用 fbc-starter 的 Kafka 配置）
}
```

**任务清单**：

- [x] 更新 `Cargo.toml` 添加依赖
- [x] 创建 `src/config.rs` 配置结构
- [x] 集成 fbc-starter 的配置系统

---

### 阶段 3：核心代码迁移（3-4 天）

#### 步骤 3.1：迁移 Sender trait 和 Notification（1 天）

**来源**：

- `flare-core/src/sender.rs` → `src/adapters/sender.rs`
- `flare-core/src/notification.rs` → `src/models/notification.rs`

**任务清单**：

- [x] 迁移 `Sender` trait
- [x] 迁移 `Notification` 结构
- [x] 简化 Notification 结构，移除不必要的字段（保持原有结构）
- [ ] 添加单元测试（后续阶段完成）

#### 步骤 3.2：迁移适配器（2 天）

**来源**：

- `flare-adapters/src/email.rs` → `src/adapters/email.rs`
- `flare-adapters/src/ali_sms.rs` → `src/adapters/sms.rs`
- `flare-adapters/src/im_feishu.rs` → `src/adapters/feishu.rs`
- `flare-adapters/src/im_dingding.rs` → `src/adapters/dingding.rs`
- `flare-adapters/src/im_wechat.rs` → `src/adapters/wechat.rs`（可选）

**任务清单**：

- [x] 迁移 Email 适配器
- [x] 迁移 SMS 适配器（阿里云）
- [x] 迁移 Feishu 适配器
- [x] 迁移 Dingding 适配器
- [x] 迁移 WeChat 适配器（可选，原始项目不存在，已创建占位文件）
- [x] 更新配置引用
- [ ] 添加单元测试（后续阶段完成）

#### 步骤 3.3：迁移枚举和错误（0.5 天）

**来源**：

- `flare-common/src/enums/channel.rs` → `src/models/channel.rs`
- `flare-common/src/enums/message.rs` → `src/models/message.rs`（简化）
- `flare-common/src/errors.rs` → `src/error.rs`（整合到 AppError）

**任务清单**：

- [x] 迁移 `ChannelType` 枚举
- [x] 迁移消息类型枚举（`FeishuMessageType`, `DingdingMessageType`）
- [x] 整合错误类型到 `AppError`（创建 NotifyError，转换为 AppError）
- [x] 更新错误处理逻辑（添加错误码系统）
- [x] 优化模块导出（所有 mod 私有，通过 pub use 导出）

#### 步骤 3.4：配置迁移（0.5 天）

**来源**：

- `flare-common/src/config.rs` → `src/config.rs`

**任务清单**：

- [x] 迁移配置结构
- [x] 改为使用 fbc-starter 的配置系统
- [x] 更新环境变量读取逻辑（实现 `from_env()` 方法）

---

### 阶段 4：Kafka 消费者集成（2 天）

#### 步骤 4.1：使用 fbc-starter 的 Kafka 功能（1 天）

**来源**：

- `flare-worker/src/handlers.rs` → `src/kafka/handler.rs`

**任务清单**：

- [x] 使用 `fbc-starter` 的 `KafkaMessageHandler` trait
- [x] 迁移消息处理逻辑
- [x] 创建 `src/kafka/handler.rs` 实现消息分发
- [x] 实现 `KafkaMessageHandler` trait
- [x] 在 `main.rs` 中使用 `Server::run` 和 `with_kafka_handler` 注册处理器
- [x] 创建 `NotificationHandlerContext` 管理所有发送器
- [x] 实现消息分发逻辑（支持 Email, SMS, Feishu, Dingding）
- [x] 支持两种消息格式（直接格式和 flare-worker 兼容格式）

#### 步骤 4.2：消息格式定义（1 天）

**来源**：

- `flare-worker/src/handlers.rs` 的 Message 结构 → `src/models/message.rs`

**任务清单**：

- [x] 定义 Kafka 消息格式（直接使用 `Notification`，兼容 flare-worker 格式）
- [x] 创建消息序列化/反序列化逻辑（支持两种格式）
- [x] 添加消息验证逻辑（字段验证和错误处理）
- [x] 实现 `parse_flare_format` 函数解析兼容格式

---

### 阶段 5：HTTP API 开发（2-3 天）

#### 步骤 5.1：定义 API 接口（1 天）

**API 设计**：

- `POST /api/v1/notifications` - 发送通知
- `GET /health` - 健康检查（由 fbc-starter 提供）
- `GET /api/v1/channels` - 获取支持的渠道列表

**任务清单**：

- [x] 设计 API 接口规范
- [x] 定义请求/响应结构（使用 fbc-starter 的 `R<T>` 响应结构）
- [x] 编写 API 文档（代码注释）

#### 步骤 5.2：实现 Handlers（1-2 天）

**任务清单**：

- [x] `src/handlers/notification.rs` - 发送通知处理器
- [x] `src/handlers/channels.rs` - 获取渠道列表处理器
- [x] 使用 Axum 的 `Handler` trait
- [x] 添加请求验证
- [x] 添加错误处理（使用 fbc-starter 的 `AppError`）
- [x] 使用 fbc-starter 提供的健康检查（无需自定义）

#### 步骤 5.3：路由配置（0.5 天）

**任务清单**：

- [x] `src/router.rs` - 定义路由
- [x] 集成到 `main.rs`
- [x] 使用 fbc-starter 提供的中间件（CORS、日志等）

---

### 阶段 6：主程序整合（1-2 天）

#### 步骤 6.1：整合 HTTP Server 和 Kafka Consumer（1 天）

**任务清单**：

- [x] 在 `main.rs` 中同时启动 HTTP Server 和 Kafka Consumer（通过 `Server::run` 统一管理）
- [x] 使用 fbc-starter 的 `Server` 统一管理（无需手动 `tokio::spawn`）
- [x] 添加优雅关闭逻辑（由 fbc-starter 提供）
- [x] 添加启动日志（由 fbc-starter 提供）

#### 步骤 6.2：错误处理和日志（1 天）

**任务清单**：

- [x] 统一错误处理（使用 fbc-starter 的 AppError，已在阶段 3 完成）
- [x] 配置日志（使用 fbc-starter 的日志系统，自动配置）
- [x] 添加错误日志记录（在 handlers 和 kafka handler 中使用 tracing）
- [ ] 添加性能监控日志（可选，后续优化）

---

### 阶段 7：测试和优化（2-3 天）

#### 步骤 7.1：单元测试（1 天）

**任务清单**：

- [x] 为模型编写单元测试（Notification, ChannelType 序列化/反序列化）
- [x] 为消息类型枚举编写测试（FeishuMessageType, DingdingMessageType）
- [x] 为错误处理编写测试（错误码、错误显示）
- [x] 为配置结构编写测试（EmailConfig 反序列化）
- [ ] 为适配器编写单元测试（需要 mock，后续优化）
- [ ] 为消息处理逻辑编写测试（需要 mock，后续优化）
- [ ] 测试覆盖率 > 80%（当前基础测试已完成）

#### 步骤 7.2：集成测试（1 天）

**任务清单**：

- [x] 测试通知消息序列化/反序列化（Kafka 消息格式）
- [x] 测试 flare-worker 格式兼容性（parse_flare_format）
- [x] 测试不同渠道的消息格式（Email, SMS, IM）
- [ ] 测试 HTTP API（需要启动服务器，后续优化）
- [ ] 测试 Kafka 消息处理（需要 Kafka 环境，后续优化）
- [ ] 测试错误场景（需要 mock，后续优化）
- [ ] 测试并发场景（需要 mock，后续优化）

#### 步骤 7.3：性能优化（1 天）

**任务清单**：

- [ ] 连接池优化
- [ ] 异步处理优化
- [ ] 错误重试机制
- [ ] 性能基准测试

---

## 三、简化策略

### 移除的内容

1. ❌ `flare-api`：直接集成到 ms-notify
2. ❌ `flare-storage`：暂不使用独立存储层
3. ❌ `flare-core` 的模板引擎：暂不实现模板功能
4. ❌ 复杂的消息类型枚举：简化为基本类型

### 简化的内容

1. ✅ **配置管理**：使用 fbc-starter 的配置系统，减少自定义配置代码
2. ✅ **错误处理**：整合到 fbc-starter 的 AppError
3. ✅ **日志**：使用 fbc-starter 的日志系统
4. ✅ **Kafka**：使用 fbc-starter 的 Kafka 功能，减少自定义代码

### 保留的核心功能

1. ✅ **Sender trait**：统一的消息发送接口
2. ✅ **所有适配器**：Email、SMS、Feishu、Dingding、WeChat
3. ✅ **异步处理**：Kafka 消息队列
4. ✅ **多渠道支持**：保持灵活性

---

## 四、实施时间表

| 阶段     | 任务             | 预计时间     | 优先级 | 状态        |
| -------- | ---------------- | ------------ | ------ | ----------- |
| 1        | 项目结构设计     | 1-2 天       | 高     | ✅ 已完成   |
| 2        | 依赖和配置       | 1 天         | 高     | ✅ 已完成   |
| 3        | 核心代码迁移     | 3-4 天       | 高     | ✅ 已完成   |
| 4        | Kafka 消费者集成 | 2 天         | 高     | ✅ 已完成   |
| 5        | HTTP API 开发    | 2-3 天       | 中     | ✅ 已完成   |
| 6        | 主程序整合       | 1-2 天       | 高     | ✅ 已完成   |
| 7        | 测试和优化       | 2-3 天       | 中     | ✅ 基本完成 |
| **总计** |                  | **12-17 天** |        |             |

---

## 五、关键决策点

### 1. 是否保留模板引擎功能？

**决策**：暂不实现，后续按需添加
**原因**：简化迁移，减少复杂度

### 2. 是否支持消息存储？

**决策**：暂不实现，后续按需添加
**原因**：先实现核心功能，存储功能可以后续扩展

### 3. 是否支持消息重试？

**决策**：实现基本的重试机制
**原因**：提高消息发送的可靠性

### 4. 是否支持批量发送？

**决策**：暂不实现，后续按需添加
**原因**：简化实现，单个消息发送已经足够

---

## 六、迁移检查清单

### 阶段 1：项目结构

- [x] 创建目录结构
- [x] 创建各模块的 `mod.rs` 文件
- [x] 定义模块导出

### 阶段 2：依赖和配置

- [x] 更新 `Cargo.toml` 添加依赖
- [x] 创建 `src/config.rs` 配置结构
- [x] 集成 fbc-starter 的配置系统

### 阶段 3：核心代码迁移

- [x] 迁移 `Sender` trait
- [x] 迁移 `Notification` 结构
- [x] 迁移 Email 适配器
- [x] 迁移 SMS 适配器
- [x] 迁移 Feishu 适配器
- [x] 迁移 Dingding 适配器
- [ ] 迁移 WeChat 适配器（可选）
- [x] 迁移枚举和错误类型（ChannelType, NotifyError, FeishuMessageType, DingdingMessageType）
- [x] 迁移配置结构（集成 fbc-starter）
- [x] 优化模块导出（所有 mod 私有，通过 pub use 导出）

### 阶段 4：Kafka 消费者集成

- [x] 实现 Kafka 消息处理器（`NotificationHandler`）
- [x] 定义消息格式（直接使用 `Notification`，支持兼容格式）
- [x] 集成 fbc-starter 的 Kafka 功能（使用 `KafkaMessageHandler` trait）
- [x] 实现消息分发逻辑（支持多种渠道）
- [x] 创建处理器上下文（`NotificationHandlerContext`）

### 阶段 5：HTTP API 开发

- [x] 实现发送通知 API（POST `/api/v1/notifications`）
- [x] 使用 fbc-starter 提供的健康检查 API（GET `/health`）
- [x] 实现渠道列表 API（GET `/api/v1/channels`）
- [x] 配置路由并集成到 `main.rs`
- [x] 使用 fbc-starter 的 `R<T>` 响应结构体
- [x] 优化代码结构（在 `NotificationHandlerContext` 中添加 `send` 方法供复用）

### 阶段 6：主程序整合

- [x] 整合 HTTP Server 和 Kafka Consumer（通过 `Server::run` 统一管理）
- [x] 统一错误处理（使用 fbc-starter 的 `AppError`）
- [x] 配置日志系统（由 fbc-starter 自动配置）

### 阶段 7：测试和优化

- [ ] 单元测试完成
- [ ] 集成测试完成
- [ ] 性能优化完成
- [ ] 文档更新完成

---

## 七、后续优化方向

### 短期优化（1-3 个月）

1. **消息模板引擎**：支持模板渲染

   - 使用 Tera 或 Handlebars
   - 支持变量替换
   - 支持条件渲染

2. **消息存储**：记录发送历史

   - 使用 MySQL 存储消息记录
   - 支持查询和统计
   - 支持消息重发

3. **消息统计**：发送成功率、失败率等
   - 使用 Redis 统计
   - 提供统计 API
   - 支持实时监控

### 中期优化（3-6 个月）

4. **限流和熔断**：防止服务过载

   - 实现限流机制
   - 实现熔断机制
   - 支持降级策略

5. **多租户支持**：支持多租户隔离

   - 租户配置隔离
   - 租户数据隔离
   - 租户权限控制

6. **Webhook 回调**：支持发送结果回调
   - 发送成功回调
   - 发送失败回调
   - 支持自定义回调地址

### 长期优化（6-12 个月）

7. **消息队列优化**：支持更多消息队列

   - 支持 RabbitMQ
   - 支持 Redis Stream
   - 支持 NATS

8. **监控和告警**：完善监控体系

   - Prometheus 指标
   - Grafana 仪表盘
   - 告警规则配置

9. **高可用性**：提升系统可用性
   - 多实例部署
   - 负载均衡
   - 故障转移

---

## 八、参考文档

### Flare 项目文件映射

| Flare 文件                          | ms-notify 目标文件           | 说明              |
| ----------------------------------- | ---------------------------- | ----------------- |
| `flare-core/src/sender.rs`          | `src/adapters/sender.rs`     | Sender trait      |
| `flare-core/src/notification.rs`    | `src/models/notification.rs` | Notification 结构 |
| `flare-adapters/src/email.rs`       | `src/adapters/email.rs`      | 邮件适配器        |
| `flare-adapters/src/ali_sms.rs`     | `src/adapters/sms.rs`        | 短信适配器        |
| `flare-adapters/src/im_feishu.rs`   | `src/adapters/feishu.rs`     | 飞书适配器        |
| `flare-adapters/src/im_dingding.rs` | `src/adapters/dingding.rs`   | 钉钉适配器        |
| `flare-common/src/enums/channel.rs` | `src/models/channel.rs`      | 渠道枚举          |
| `flare-common/src/config.rs`        | `src/config.rs`              | 配置结构          |
| `flare-worker/src/handlers.rs`      | `src/kafka/handler.rs`       | 消息处理          |
| `flare-worker/src/main.rs`          | `src/kafka/consumer.rs`      | Kafka 消费者      |

### 依赖版本

- `lettre`: 0.11
- `hmac`: 0.12
- `sha2`: 0.10
- `reqwest`: 使用 workspace 版本
- `base64`: 使用 workspace 版本

---

## 九、注意事项

1. **配置管理**：确保所有配置都通过 fbc-starter 的配置系统加载
2. **错误处理**：统一使用 fbc-starter 的 AppError，避免自定义错误类型
3. **日志记录**：使用 fbc-starter 的日志系统，保持日志格式一致
4. **异步处理**：确保所有 I/O 操作都是异步的
5. **资源清理**：确保连接池、客户端等资源正确释放
6. **测试覆盖**：确保关键功能都有测试覆盖

---

## 十、更新日志

- **2025-01-XX**：创建迁移计划文档
- 待更新...

---

## 十一、联系方式

如有问题或建议，请联系项目维护者。
