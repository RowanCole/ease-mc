# EaseMC (client)

基于 **Tauri 2 + React 18 + TypeScript + Zustand + Rust** 的 Minecraft 1.21.1 离线启动器（项目总览见根目录 [README.zh.md](../README.zh.md)）。面向新手玩家:

- 首次启动自动下载游戏与 Java 运行环境,无需手动安装
- 使用 BMCLAPI 国内镜像加速下载,失败自动回退官方源
- 一键启动 / 结束游戏,支持跨平台(macOS / Windows)
- 内置 AI 游戏助手(DeepSeek),帮助新手度过新手期

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | React 18 + TypeScript + Vite 6,`@tauri-apps/api`、`lucide-react`、`zustand`、`marked` |
| 后端 | Tauri 2 + Rust Cargo workspace:`mc-core` / `mc-downloader` / `mc-launcher` / `mc-assistant` |
| AI | `async-openai` 调用 DeepSeek 流式对话 |

后端按领域拆分为四个 crate,`src-tauri/src` 仅作为应用壳注册命令并转发:

| crate | 职责 |
|---|---|
| `mc-core` | 运行路径、配置读写、版本清单(`game.json` 编译期嵌入) |
| `mc-downloader` | HTTP 传输(BMCLAPI 镜像回退、并发、进度事件)、游戏文件与 JRE 安装 |
| `mc-launcher` | 启动 / 结束 Minecraft 进程 |
| `mc-assistant` | DeepSeek 流式 AI 助手 |

## 目录结构

```
client/
├── index.html
├── package.json
├── tsconfig.json
├── vite.config.ts
├── src/                      # React + TypeScript 前端
│   ├── main.tsx              # React 入口
│   ├── App.tsx               # 根组件:布局 + game-exited 监听
│   ├── App.css               # 全局样式
│   ├── types.ts              # 共享 TypeScript 类型
│   ├── constants.ts          # 常量(gameInfo / quickPrompts / isTauri)
│   ├── utils.ts              # 工具函数(errorText)
│   ├── assets/
│   ├── stores/               # Zustand 状态管理
│   │   ├── uiStore.ts        # 视图切换 + 游戏助手面板开关
│   │   ├── toastStore.ts     # 弹窗通知
│   │   ├── gameStore.ts      # 游戏状态 / 下载 / 启动逻辑
│   │   └── chatStore.ts      # AI 助手流式对话
│   └── components/           # 函数组件
│       ├── LauncherView.tsx  # 普通模式页面(背景 + 欢迎区 + 启动卡片)
│       ├── AdvancedMode.tsx  # 高级模式页面(独立背景与布局,占位待开发,点击左上角图标进入)
│       ├── Scene.tsx         # 背景场景(普通模式)
│       ├── TopBar.tsx        # 顶部栏(品牌 + 游戏助手入口 + 模式切换)
│       ├── HeroSection.tsx   # 欢迎区
│       ├── LaunchCard.tsx    # 启动卡片(状态文案区)
│       ├── LaunchButton.tsx  # 启动/下载按钮(含下载波浪动画)
│       ├── ChatPanel.tsx     # AI 助手抽屉(消息列表 + 输入框)
│       └── ToastStack.tsx    # 弹窗通知
└── src-tauri/                # Tauri + Rust 后端(Cargo workspace)
    ├── tauri.conf.json
    ├── config.json           # 运行时配置(见下文)
    ├── capabilities/default.json
    ├── src/                  # 应用壳:仅窗口入口与命令转发
    │   ├── main.rs           # 入口,初始化日志与 dotenv
    │   └── lib.rs            # #[tauri::command] 定义,转发到各领域 crate
    └── crates/
        ├── mc-core/          # 基础能力:路径(paths)、配置(config)、清单解析(manifest)
        │   └── src/game.json # Minecraft 1.21.1 版本清单(编译期嵌入)
        ├── mc-downloader/    # 下载域:net(传输) / download(编排) / jre(运行环境) / progress(进度)
        ├── mc-launcher/      # 启动域:game.rs(启动 / 结束游戏进程)
        └── mc-assistant/     # AI 助手域:chat.rs(DeepSeek 流式对话)
```

## 快速开始

前置要求:[Node.js](https://nodejs.org/)、[Rust](https://rustup.rs/)(含 Tauri CLI 所需的系统依赖)。

```bash
# 安装前端依赖
npm install

# 启动 Tauri 开发环境(会同时启动 Vite 与 Rust)
npm run tauri dev

# 仅前端热更新(浏览器预览,部分功能不可用)
npm run dev

# 构建发布版本
npm run tauri build
```

## 配置说明

`src-tauri/config.json`(打包后作为资源,首次运行会复制到**可执行文件同级目录**,与 `game/` 一起构成便携式布局):

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
| `gameIsInstalled` | 游戏是否已安装,启动器据此决定是否自动下载 |
| `macJrePath` / `winJrePath` | 对应平台 JRE 21 的下载地址(zip / tar.gz) |
| `serverUrl` | 预留字段,当前代码未使用 |

## AI 助手配置

在 `client/src-tauri/` 目录下创建 `.env`(`npm run tauri dev` 时 Rust 进程的工作目录),填入 DeepSeek API Key(请勿提交到版本库;Key 缺失仅影响 AI 功能,下载与启动不受影响):

```
DEEPSEEK_API_KEY=sk-xxxxxxxx
```

## 常用命令(后端)

```bash
cd src-tauri

# 运行 Rust 测试
# 注意:mc-downloader 的集成测试会访问真实网络,下载几百 MB 的游戏文件与 JRE
cargo test

# 被 #[ignore] 标记的测试(DeepSeek 对话,需要 API Key 与网络)
cargo test -- --ignored --nocapture
```

## 已知限制

- 仅支持原版 Minecraft 1.21.1,暂不支持 Mod
- 离线模式,不支持正版账号登录
- Apple Silicon 上通过 Rosetta 运行 x64 JRE(原生 arm64 适配在开发计划中)
- 下载文件暂不做 SHA1 完整性校验(开发计划中)

## 推荐 IDE 配置

- [VS Code](https://code.visualstudio.com/) + [ES7+ React/Redux](https://marketplace.visualstudio.com/items?itemName=dsznajder.es7-react-js-snippets) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
