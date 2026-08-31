<div align="center">

<img src="client/EaseMC.png" width="120" alt="EaseMC" />

# EaseMC

面向新手的零配置、一键式 Minecraft 启动器。

[![Tauri](https://img.shields.io/badge/Tauri_2-FFC131?style=flat-square&logo=tauri&logoColor=black)](https://tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte_5-FF3E00?style=flat-square&logo=svelte&logoColor=white)](https://svelte.dev)
[![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue?style=flat-square)](#)

[English](README.md) | **简体中文**

</div>

## 项目简介

EaseMC 是一个开源的 Minecraft 启动器，专注于让新手玩家以最低门槛开始游戏：无需手动安装 Java、无需研究版本与安装流程——点击「下载游戏」按钮，启动器会自动完成游戏下载、运行环境安装，再一键进入游戏。

启动器还内置了 AI 游戏助手（由 DeepSeek 驱动），随时解答生存、合成、探索等新手问题。

> EaseMC 当前面向 Minecraft **1.21.1**（原版、离线模式），界面为中文。

## 功能特性

- **零配置** — 点击「下载游戏」按钮，自动下载全部游戏文件与 Java 21 运行环境（Azul Zulu），无需手动安装任何东西
- **高速可靠的下载** — 8 并发下载 + 实时进度；默认走 BMCLAPI 国内镜像加速，失败自动回退 Mojang 官方源
- **一键启动 / 结束** — 单按钮控制游戏开始与退出，启动器代为管理游戏进程
- **内置 AI 游戏助手** — DeepSeek 流式对话，预置启动器相关问答（安装方式、文件位置、故障排查），支持 Markdown 渲染与快捷提问
- **跨平台** — 支持 macOS 与 Windows
- **轻量** — 基于 Tauri 2 构建：小巧的 Rust 内核 + Web UI

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | Svelte 5 + Vite 6，`@tauri-apps/api`、`lucide-svelte`、`marked` |
| 后端 | Tauri 2 + Rust（`tokio`、`reqwest`、`zip` / `flate2` / `tar`） |
| AI | `async-openai` 调用 DeepSeek 流式对话 |

## 快速开始

### 前置要求

- [Node.js](https://nodejs.org/)
- [Rust](https://rustup.rs/)（含 Tauri 所需的平台系统依赖，参见 [Tauri 前置依赖指南](https://tauri.app/start/prerequisites/)）

### 开发调试

```bash
cd client

# 安装前端依赖
npm install

# 启动 Tauri 开发环境（同时启动 Vite 与 Rust）
npm run tauri dev

# 仅前端热更新（浏览器预览，后端功能不可用）
npm run dev
```

### 构建

```bash
npm run tauri build
```

安装包输出至 `client/src-tauri/target/release/bundle/`。

### 后端测试

```bash
cd client/src-tauri

# 快速测试
cargo test

# 依赖真实网络的测试（标记为 #[ignore]，需显式执行）
cargo test -- --ignored --nocapture
```

## 配置说明

### 启动器配置 — `client/src-tauri/config.json`

打包时作为资源附带，首次运行会复制到系统应用配置目录。

```json
{
  "gameIsInstalled": "false",
  "macJrePath": "https://cdn.azul.com/zulu/.../zulu21-macosx_x64.tar.gz",
  "winJrePath": "https://cdn.azul.com/zulu/.../zulu21-win_x64.zip",
  "serverUrl": "http://localhost:3000"
}
```

| 字段 | 说明 |
|---|---|
| `gameIsInstalled` | 游戏是否已安装，启动器据此决定是否自动下载 |
| `macJrePath` / `winJrePath` | 对应平台 JRE 21 的下载地址（tar.gz / zip） |
| `serverUrl` | 预留字段，当前未使用 |

### AI 助手

在 `client/` 目录下创建 `.env` 文件（即 `npm run tauri dev` 的工作目录），填入 DeepSeek API Key：

```
DEEPSEEK_API_KEY=sk-xxxxxxxx
```

请勿提交该文件。Key 缺失时仅 AI 助手不可用，游戏下载与启动不受影响。

## 工作原理

1. **首次运行** — 启动器检查安装状态；点击「下载游戏」按钮后，并发下载全部游戏文件（BMCLAPI 镜像优先，失败回退 Mojang 官方源），随后下载并解压 Java 21 运行环境。
2. **启动游戏** — Rust 后端从 `game/.minecraft/libraries` 收集 classpath 并拼入客户端 jar，按平台以正确的本地库参数拉起 `java` 进程。
3. **文件位置** — 游戏文件存放在可执行文件旁的 `game/` 目录中，绿色便携、自包含。

## 目录结构

```
client/
├── index.html
├── package.json
├── vite.config.js
├── src/                      # Svelte 前端
│   ├── main.js
│   ├── App.svelte            # 根组件：状态编排 + 业务逻辑
│   ├── App.css               # 全局样式
│   ├── constants.js          # 常量（gameInfo / quickPrompts / isTauri）
│   ├── assets/
│   └── components/           # 组件化 UI
│       ├── LauncherView.svelte # 普通模式页面（背景 + 欢迎区 + 启动卡片）
│       ├── AdvancedMode.svelte # 高级模式页面（独立背景与布局，占位待开发，点击左上角图标进入）
│       ├── Scene.svelte      # 背景场景（普通模式）
│       ├── TopBar.svelte     # 顶部栏（品牌 + 游戏助手入口 + 模式切换）
│       ├── HeroSection.svelte# 欢迎区
│       ├── LaunchCard.svelte # 启动卡片（状态文案区）
│       ├── LaunchButton.svelte # 启动/下载按钮（含下载波浪动画）
│       ├── ChatPanel.svelte  # AI 助手抽屉（消息列表 + 输入框）
│       └── ToastStack.svelte # 弹窗通知
└── src-tauri/                # Tauri + Rust 后端
    ├── tauri.conf.json
    ├── config.json           # 运行时配置（见上文）
    ├── capabilities/default.json
    └── src/
        ├── main.rs           # 入口：日志与 dotenv 初始化
        ├── lib.rs            # Tauri 命令注册
        ├── game.rs           # 启动 / 结束 Minecraft 进程
        ├── download.rs       # 游戏文件下载（镜像加速、并发、进度事件）
        ├── jre.rs            # 下载并解压 JRE
        ├── chat.rs           # DeepSeek 流式 AI 助手
        ├── config.rs         # 读取 / 写入配置
        └── game.json         # Minecraft 1.21.1 版本清单（编译期嵌入）
```

## 已知限制

- 仅支持原版 Minecraft 1.21.1，暂不支持 Mod
- 离线模式，不支持微软账号登录
- Apple Silicon 上通过 Rosetta 运行 x64 JRE（暂未原生支持 arm64）
- 界面目前仅有中文（欢迎贡献英文翻译）

## 开发计划

- 支持多个游戏版本
- 支持 Mod 加载器（Fabric / Forge）
- 原生支持 Apple Silicon
- 下载文件 SHA1 完整性校验
- 界面英文国际化

## 参与贡献

欢迎提交 Issue 与 Pull Request。本地开发环境搭建请参考[快速开始](#快速开始)。

## 版权声明 / Copyright

本项目为第三方开源 Minecraft 启动器工具，仅供学习、研究、非商业用途使用。
项目所有代码由本人独立开发，遵循对应开源协议开源共享。

Minecraft 为 Mojang AB 及 Microsoft 旗下注册商标。
本项目与 Mojang、Microsoft 无任何关联、未经官方授权。
项目内所有优化模组、资源归原作者所有。

禁止未经授权的商业售卖、二次封装牟利、虚假署名发布。
