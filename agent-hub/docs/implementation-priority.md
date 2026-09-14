# AgentHub 功能模块实现优先级

> 依据：依赖方向 `server → gateway → auth/registry/agent → storage → common`，遵循「最小可运行 → 核心闭环 → 安全 → 生产化」的增量路线。
> 当前骨架中所有 `todo!()` 均已标注在对应模块条目中。

## 总览

| 优先级 | 主题 | 模块 | 产出标志 |
|---|---|---|---|
| P0 | 服务可运行 | config、server、gateway(路由) | `cargo run -p server` 启动成功，`/healthz` 返回 200 |
| P1 | 核心 MVP（内存闭环） | registry、gateway(Agent API)、agent、gateway(invoke) | 注册 Agent → 列表可见 → 调用返回 |
| P2 | 安全与身份 | auth、gateway(middleware) | 无凭证请求被拒 401，JWT/API Key 可通过 |
| P3 | 流式调用 | agent、gateway | `/v1/invoke` 支持 SSE/chunked 透传 |
| P4 | 生产化底座 | observability、storage | `/metrics` 暴露；Postgres/Redis 后端可切换 |
| P5 | 周边与增强 | sdk、限流/配额 | 外部客户端可用 SDK 接入 |

---

## P0 — 服务可运行（先让进程活着）

1. **config：三层配置加载** — [loader.rs](../crates/config/src/loader.rs)
   - 实现 `load()` / `load_from()`：默认值 < 配置文件（`AGENTHUB_CONFIG_FILE`，toml/yaml/json）< 环境变量 `AGENTHUB_*`。
   - 工具：`config_rs`（crates.io 的 `config` crate）。
   - 验收：单测覆盖三层覆盖顺序；`main.rs` 读取 `settings.server.host/port` 正确。
2. **server：HTTP 监听** — [main.rs](../crates/server/src/main.rs)
   - 移除 `todo!()`：`axum::serve(listener, router)` 监听 `settings.server.host:port`，加优雅停机（ctrl_c）。
   - 验收：`cargo run -p server` 常驻运行，Ctrl+C 干净退出。
3. **gateway：基础路由骨架** — [router.rs](../crates/gateway/src/router.rs)
   - `build_router()`：`/healthz` + tower-http（trace / timeout / cors）。
   - 验收：`GET /healthz` 200。
4. **gateway：请求 ID 中间件** — [middleware.rs](../crates/gateway/src/middleware.rs)
   - 生成/透传 `X-Request-Id`，写入 extensions 与响应头。

## P1 — 核心 MVP：注册 → 发现 → 调用（全内存版）

5. **registry：内存注册中心** — [registry.rs](../crates/registry/src/registry.rs)
   - `InMemoryRegistry` 实现五方法：register（生成 AgentId + 租约）、renew、deregister、list_online（过滤过期租约）、get。
   - 数据结构：`DashMap<AgentId, RegisteredAgent>`，租约含过期时间戳。
   - 验收：注册 → list_online 可见 → 不续约等过期后消失。
6. **gateway：Agent 管理 API** — `/v1/agents`（POST 注册 / DELETE 注销 / GET 列表与详情）
   - 请求体校验（validator），错误统一映射 `AgentHubError` → HTTP 状态码。
7. **agent：转发客户端（非流式）** — [client.rs](../crates/agent/src/client.rs)
   - `invoke()`：查注册中心取 endpoint，reqwest POST 转发 `InvocationRequest`，超时与错误处理。
   - `health()`：探测 Agent 健康端点返回 `AgentStatus`。
   - 测试：wiremock 模拟 Agent 端。
8. **gateway：调用入口** — `/v1/invoke`
   - 校验请求 → 选 Agent → `AgentClient::invoke()` → 返回 `InvocationResponse`。
   - **P1 里程碑验收**：端到端脚本完成「注册 Agent（wiremock 假后端）→ list → invoke 拿到响应」。

## P2 — 安全与身份

9. **auth：JWT 签发/校验** — [jwt.rs](../crates/auth/src/jwt.rs)
   - `JwtSigner::sign` / `JwtVerifier::verify`（jsonwebtoken，HS256，校验 exp/iss/aud）。
10. **auth：API Key** — [api_key.rs](../crates/auth/src/api_key.rs)
    - Key 生成（随机 + 前缀）、argon2 哈希存储、常量时间比对、`Principal` 解析。
11. **gateway：认证中间件接入** — [middleware.rs](../crates/auth/../gateway/src/middleware.rs)
    - 提取 `Authorization: Bearer` / `X-Api-Key` → 校验 → 注入 `Principal` 到 extensions，失败 401。
    - `/healthz` 与 Agent 自注册端点白名单放行。
    - **P2 验收**：无凭证 401；有效 JWT / API Key 全链路打通（含多租户 `TenantId` 隔离查询）。

## P3 — 流式调用

12. **agent：流式转发** — `invoke_stream()`
    - reqwest `bytes_stream` → axum `Sse`/`Body` 透传，保留背压；Agent 健康探测失败时提前熔断。
13. **gateway：`/v1/invoke` 流式分支**
    - `InvocationRequest.stream == true` 时切换 SSE 响应；心跳注释帧保活。

## P4 — 生产化底座

14. **observability：指标** — [metrics.rs](../crates/observability/src/metrics.rs)
    - Prometheus exporter 挂 `/metrics`；基础指标：请求计数/时延、Agent 在线数、调用错误率。
15. **storage：Postgres 仓储** — [postgres.rs](../crates/storage/src/postgres.rs) + sqlx migrate
    - `agents` 表、租约字段、索引；实现与 `AgentRegistry` 对接的仓储 trait。
16. **storage：Redis** — [redis 层]
    - 租约 TTL / 心跳续约（`SETEX`）、分布式请求 ID 去重（可选）。
17. **server：按 `storage.backend` 装配**
    - `memory | postgres | redis` 切换真实实现，替换 `InMemoryRegistry`。

## P5 — 周边与增强

18. **sdk：客户端 SDK** — [sdk/client.rs](../crates/sdk/src/client.rs)
    - 封装注册/发现/调用（含流式），`SdkError` 统一错误；不依赖内部 crate。
19. **增强项（按需）**
    - 限流（tower `governor`/令牌桶）、按租户配额；
    - Agent 负载均衡 / 能力路由（按 `capabilities` 选择）；
    - 重试与熔断（tower middleware）；
    - 配置热更新、管理后台 API。

---

## 依赖与并行建议

- **P0 内部串行**：config → server → gateway 路由（main.rs 依赖前两者）。
- **P1 可并行**：registry（5）与 agent client（7）互不依赖，可并行；gateway API（6、8）依赖两者完成。
- **P2 依赖 P1** 的路由结构；JWT（9）与 API Key（10）可并行。
- **P4 的 storage（15、16）只依赖 common**，可与 P2/P3 并行推进，最后在 server 装配层汇合。
