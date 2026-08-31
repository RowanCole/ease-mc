# MC Starter

基于 **Tauri 2 + React 18 + TypeScript + Zustand + Rust** 的 Minecraft 1.21.1 离线启动器。面向新手玩家:

- 首次启动自动下载游戏与 Java 运行环境,无需手动安装
- 使用 BMCLAPI 国内镜像加速下载,失败自动回退官方源
- 一键启动 / 结束游戏,支持跨平台(macOS / Windows)
- 内置 AI 游戏助手(DeepSeek),帮助新手度过新手期

## 技术栈

| 层 | 技术 |
|---|---|
| 前端 | React 18 + TypeScript + Vite 6,`@tauri-apps/api`、`lucide-react`、`zustand`、`marked` |
| 后端 | Tauri 2 + Rust(`tokio`、`reqwest`、`zip` / `flate2` / `tar`) |
| AI | `async-openai` 调用 DeepSeek 流式对话 |

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
└── src-tauri/                # Tauri + Rust 后端
    ├── tauri.conf.json
    ├── config.json           # 运行时配置(见下文)
    ├── capabilities/default.json
    └── src/
        ├── main.rs           # 入口,初始化日志与 dotenv
        ├── lib.rs            # 注册 Tauri 命令
        ├── game.rs           # 启动 / 结束 Minecraft 进程
        ├── download.rs       # 下载游戏文件(镜像加速、并发、进度事件)
        ├── jre.rs            # 下载并解压 JRE
        ├── chat.rs           # DeepSeek 流式 AI 助手
        ├── config.rs         # 读取 / 写入配置
        └── game.json         # Minecraft 1.21.1 版本清单(编译期嵌入)
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

`src-tauri/config.json`(打包后作为资源,首次运行会复制到系统应用配置目录):

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

在项目根目录创建 `.env` 并提供 DeepSeek API Key(请勿提交到版本库):

```
DEEPSEEK_API_KEY=sk-xxxxxxxx
```

## 常用命令(后端)

```bash
cd src-tauri

# 运行 Rust 测试(需要真实网络,部分测试默认 #[ignore],需显式执行)
cargo test
cargo test -- --ignored --nocapture
```

## 推荐 IDE 配置

- [VS Code](https://code.visualstudio.com/) + [ES7+ React/Redux](https://marketplace.visualstudio.com/items?itemName=dsznajder.es7-react-js-snippets) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
