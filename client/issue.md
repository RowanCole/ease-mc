# MC Starter — 待修复问题清单

> 生成日期:2026-08-13
> 依据:对 `src/` 前端与 `src-tauri/src/` 后端代码的完整审查

## 优先级总览

| 优先级 | 数量 | 说明 |
|---|---|---|
| **P0** | 3 | 致命 / 安全,上线前必须修复 |
| **P1** | 4 | 重要功能缺陷与健壮性问题,建议尽快修复 |
| **P2** | 6 | 体验优化与代码清理 |

---

## P0 — 必须立即修复

### P0-1 `.env`(含 DEEPSEEK_API_KEY)被打包进发布产物 — 密钥泄露
- **位置**:`src-tauri/.env`;已确认存在于 `target/release/bundle/macos/mc-starter.app/Contents/Resources/.env`
- **问题**:DeepSeek API Key 被复制进了正式安装包,任何人解包即可提取,可能被滥用产生费用。
- **建议**:
  1. 排查 `.env` 进入 bundle 的途径(确认 `tauri.conf.json` 的 `bundle.resources` 只保留 `config.json`,并检查 `build.rs` / 打包脚本);
  2. 增加 `.gitignore` 忽略 `.env`;
  3. 线上密钥应通过应用运行时注入(如环境变量/系统钥匙串),而非打包进应用。

### P0-2 `main.rs` 中 `dotenv().expect()` 在无 `.env` 时直接崩溃 — 应用无法启动
- **位置**:`src-tauri/src/main.rs` 第 9 行 `dotenv().expect("环境加载失败");`
- **问题**:`dotenv::dotenv()` 从**当前工作目录**查找 `.env`,找不到文件时返回 `Err`,`expect()` 直接 panic。`cargo run`(cwd=src-tauri)能通过,但:
  - `npm run tauri dev` 时 cwd 为 `client/`,找不到 `src-tauri/.env`;
  - 打包后的 `.app` 启动时 cwd 更不可控。
  结果是**发布版一启动就崩溃**,连下载/启动游戏都无法使用。
- **建议**:改为 `let _ = dotenv();` 或 `dotenv().ok();`。API Key 缺失只应影响 AI 功能(`chat.rs` 已有明确报错提示),不应导致整个程序 panic。

### P0-3 游戏目录依赖进程 cwd,打包后定位不到 `game/`
- **位置**:`src-tauri/src/game.rs`(`std::env::current_dir().unwrap().join("game")`)、`download.rs`(`Path::new("game")`)、`jre.rs`
- **问题**:所有游戏文件路径都以**运行时工作目录**为基准。Tauri 打包后 cwd 不可控,`game/.minecraft`、`game/java` 极可能落在错误位置或根本不存在,导致下载/启动全部失败;且 `.unwrap()` 有 panic 风险。
- **建议**:改用 `app.path().app_data_dir()`(或与可执行文件同目录的固定路径)作为游戏根目录,启动时 `create_dir_all` 确保存在,并把该路径贯穿 `download.rs` / `jre.rs` / `game.rs`。

---

## P1 — 重要问题

### P1-1 AI 助手对话历史无限增长
- **位置**:`src-tauri/src/chat.rs`(`CHAT_HISTORY` 全局 `Vec`)
- **问题**:每次对话都会把用户消息和助手回复永久追加到全局历史,永不裁剪。长时间使用后单次请求的 token 数线性膨胀,导致费用上升、响应变慢,最终可能超出模型上下文上限。
- **建议**:对历史做滑动窗口(如仅保留最近 N 轮),或按 token 数截断;错误中断时也要保证用户消息与助手回复成对入栈(见 P1-4 关联)。

### P1-2 下载文件缺少 SHA1 完整性校验
- **位置**:`src-tauri/src/download.rs`(`download_file` / `download_files_concurrent`)
- **问题**:下载后不校验哈希。镜像源异常(返回 404 页、被劫持、文件损坏)时会静默保存坏文件,启动游戏时出现难以排查的崩溃。`game.json` 中每个 artifact 都已带 `sha1` 与 `size`,但代码完全未使用。
- **建议**:下载完成后按清单中的 `sha1` 校验,失败先回退官方源,再失败则报错重试;同时利用 `size` 做提前判断。

### P1-3 JRE 解压阻塞 tokio 运行时
- **位置**:`src-tauri/src/jre.rs`(`extract_archive` 用同步 `zip.extract` / `tar.unpack`)
- **问题**:同步解压在 async 函数中执行,会阻塞 tokio worker 线程,期间下载进度事件、命令响应全部卡住(几百 MB 的 JRE 解压尤其明显)。
- **建议**:用 `tokio::task::spawn_blocking` 包裹解压逻辑,或改用异步解压。

### P1-4 前端开发模式自动弹出假通知(调试残留)
- **位置**:`src/App.svelte` 的 `onMount`(DEV 下 1s 后依次弹"下载失败 / 下载完成 / 安装环境"三条通知)
- **问题**:纯调试用假通知,开发时会误导状态判断、污染截图与 UI 调试。
- **建议**:删除该段,或收敛为显式开启的调试开关。

---

## P2 — 改进与清理

### P2-1 缺少 `.gitignore`
- **位置**:项目根目录
- **问题**:没有 `.gitignore`,`node_modules/`、`dist/`、`src-tauri/target/`、`.env` 都有被误提交的风险(与 P0-1 直接相关)。
- **建议**:补充标准 Tauri + Svelte 的 `.gitignore`(node_modules、dist、target、.env 等)。

### P2-2 `{@html marked.parse(...)}` 渲染 AI 输出存在 XSS 隐患
- **位置**:`src/App.svelte`(`chat-bubble markdown` 消息渲染)
- **问题**:直接 `@html` 渲染 AI 回复。当前 DeepSeek 输出基本可信,但属于潜在注入面(提示词注入可能让 AI 输出恶意 HTML/脚本)。
- **建议**:接入 `DOMPurify` 做净化后再渲染,或至少移除风险标签。

### P2-3 解压阶段启动按钮未禁用
- **位置**:`src/App.svelte`(`disabled={isDownloading}`)
- **问题**:`isExtracting` 阶段 `isDownloading` 为 `false`,按钮可点击;此时点击会触发 `launch_game`,而 `game/java` 尚未就绪,可能启动失败。
- **建议**:改为 `disabled={isDownloading || isExtracting}`。

### P2-4 清理无关残留目录 `src-tauri/__MACOSX/`
- **位置**:`src-tauri/__MACOSX/`(内含无关的 Python 作业文件,是 zip 解压残留)
- **问题**:与项目无关的垃圾文件,污染仓库与打包。
- **建议**:删除该目录。

### P2-5 macOS 架构(x64 / arm64)未适配
- **位置**:`src-tauri/config.json`(`macJrePath` 固定为 `macosx_x64`)、`jre.rs`
- **问题**:Apple Silicon 上会以 Rosetta 运行 x64 JRE,性能与兼容性不理想;`game.json` 中 natives 也区分了 `natives-macos-arm64`,但下载与 JRE 选择未按架构区分。
- **建议**:按 `std::env::consts::ARCH` 选择对应架构的 JRE 下载地址,并确保 natives 匹配。

### P2-6 `serverUrl` 配置项未使用
- **位置**:`src-tauri/config.json`
- **问题**:`serverUrl` 已预留但代码中无任何引用,易造成误解。
- **建议**:在文档标注"预留",或删除;若后续接服务端再启用。

---

## 建议修复顺序

1. **P0-2**(panic 崩溃)→ 2. **P0-1**(密钥泄露)→ 3. **P0-3**(路径 cwd 依赖)→ 4. **P1** 各项 → 5. **P2** 清理项。
