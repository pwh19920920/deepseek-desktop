# DeepSeek Harness Desktop

<div align="center">

**将 [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) AI 工作流框架打包为原生桌面应用**

[![CI](https://github.com/Nonnetta/deepseek-desktop/actions/workflows/ci.yml/badge.svg)](https://github.com/Nonnetta/deepseek-desktop/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

</div>

---

## 免责声明 / Disclaimer

> **🚨 本项目是第三方爱好者开发的桌面壳层，与 DeepSeek 官方无关。**
>
> - 本应用是**非官方**的第三方项目，由社区开发者独立维护，**不代表 DeepSeek 官方**，与 DeepSeek（深度求索）公司**无任何关联或隶属关系**。
> - 本应用仅提供一个运行 [deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) 的桌面环境，**本身不提供任何 AI 模型、API 密钥或服务**。用户需自行准备和遵守所使用的 AI 服务条款。
> - 本项目的所有商标、品牌名称均为其各自所有者的财产。DeepSeek 名称和标识归 DeepSeek（深度求索）所有。
> - **使用者自行承担一切风险**。开发者不对因使用本软件而产生的任何直接或间接损失承担责任，包括但不限于：数据丢失、服务中断、法律合规性问题、或任何第三方索赔。
> - 用户使用本软件进行的任何行为，包括但不限于模型调用、内容生成、数据处理等，**均与开发者无关**，用户应自行确保其使用方式符合相关法律法规及服务条款。

---

## 概述

DeepSeek Harness Desktop 是一个 **Tauri 2** 桌面应用，将 DeepSeek 官方的 [deepseek-harness](https://github.com/deepseek-ai/deepseek-harness)（简称 dsh）包装为原生桌面体验。

它不是一个聊天机器人客户端，而是一个 **AI 工作流引擎的桌面壳层** — 运行 dsh 的 Web UI，提供文件系统访问、系统通知、本地文件打开等原生能力桥接。

---

## 特点

1. **原生轻量** — 基于 Tauri 2，使用系统原生 WebView，不捆绑 Chromium，包体积小、内存占用低
2. **跨平台** — 支持 macOS (ARM64/x64)、Windows (x64) 和 Linux (x64/ARM64)
3. **官方 dsh 内核** — 直接使用 `@deepseek-ai/dsh` npm 包作为依赖，版本管理透明，`pnpm update` 即可升级
4. **自包含** — 构建时自动下载对应平台的便携 Node.js 二进制，拷贝并裁剪 dsh 依赖，最终产物开箱即用，用户无需安装任何运行时
5. **构建优化** — 自动移除 `typescript`、`vite`、`esbuild` 等构建时依赖，只保留当前平台的 `node-pty` prebuild，减少包体积
6. **原生 OS 集成** — 通过 Tauri 插件提供系统通知、文件对话框、默认程序打开文件等能力
7. **优雅的启动体验** — 启动时展示加载动画，实时推送 `loading → ready / error` 状态
8. **干净关闭** — 窗口关闭时自动终止 dsh 侧边进程，无残留

---

## 快速开始

```bash
# 1. 安装依赖 + 初始化环境
pnpm bootstrap

# 2. 开发模式（带热重载）
pnpm dev

# 3. 构建生产包
pnpm build

# 4. 清理构建产物
pnpm clean
```

## 前置条件

| 依赖 | 版本要求 |
|------|---------|
| [Rust](https://rustup.rs/) | ≥ 1.77 |
| Node.js | ≥ 22.19 或 ≥ 24 |
| [pnpm](https://pnpm.io/) | ≥ 11.7 |
| Tauri 系统依赖 | 见 [Tauri 文档](https://tauri.app/start/prerequisites/) |

### macOS

```bash
xcode-select --install
```

### Windows

- [Microsoft Visual Studio C++ 构建工具](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)（Windows 10 以上系统已内置）

### Linux

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## 命令说明

| 命令 | 说明 |
|------|------|
| `pnpm bootstrap` | 安装 pnpm 依赖 + 初始化环境 |
| `pnpm dev` | 启动 Tauri 开发模式 |
| `pnpm build` | 构建当前平台 |
| `pnpm build macos` | 构建 macOS (ARM64) |
| `pnpm build windows` | 构建 Windows (x64) |
| `pnpm build linux` | 构建 Linux (x64) |
| `pnpm build all` | 构建所有平台 |
| `pnpm clean` | 清理所有构建产物 |

## 安装包说明

### macOS

构建产物为 `.dmg` 和 `.app`，位于 `dsh-app-desktop/src-tauri/target/aarch64-apple-darwin/release/bundle/`。

> **⚠️ macOS 注意：** 由于未使用 Apple Developer 账号签名，macOS Gatekeeper 可能会提示"已损坏"。
> 安装后运行以下命令即可解决：
> ```bash
> sudo xattr -rd com.apple.quarantine /Applications/DeepSeek\ Harness\ Desktop.app
> ```

### Windows

构建产物为 `.msi` 或 `.exe`，位于 `dsh-app-desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/`。

### Linux

构建产物为 `.deb` 和 `.AppImage`，位于 `dsh-app-desktop/src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/`。

## 架构

```
┌─────────────────────────────────────────────┐
│  Tauri 桌面应用                               │
│  ┌────────────────┐  ┌────────────────────┐  │
│  │  WebView        │  │  Rust 后端          │  │
│  │  (React SPA)    │◄─┤  ┌──────────────┐  │  │
│  │                 │  │  │ dsh 生命周期  │  │  │
│  │ 加载/错误/就绪  │  │  │ - 启动/停止   │  │  │
│  │  三种状态 UI    │  │  │ - 端口发现    │  │  │
│  └────────────────┘  │  └──────┬───────┘  │  │
│                      │         │          │  │
│  ┌────────────────┐  │  ┌──────┴───────┐  │  │
│  │  原生能力桥接   │  │  │  Node.js     │  │  │
│  │ - 文件选择器    │  │  │  侧边进程    │  │  │
│  │ - 系统通知      │  │  │  (dsh web)  │  │  │
│  │ - 路径打开      │  │  └─────────────┘  │  │
│  └────────────────┘  │                   │  │
│  ┌────────────────┐  │                   │  │
│  │  Tauri 插件    │  │                   │  │
│  │ - shell        │  │                   │  │
│  │ - notification │  │                   │  │
│  │ - dialog       │  │                   │  │
│  └────────────────┘  │                   │  │
└─────────────────────────────┘
```

### 工作流程

1. **启动时**：Rust 后端通过 `tauri-plugin-shell` 启动 Node.js 侧边进程，运行 `dsh web --port 0`
2. **端口发现**：从侧边进程 stdout 使用正则匹配动态分配的端口号（支持多种输出格式）
3. **加载 UI**：WebView 导航到 `http://127.0.0.1:<port>` 加载 dsh 网页界面
4. **状态管理**：通过 Tauri 事件系统向前端实时推送 `loading → ready / error` 状态
5. **关闭时**：自动终止侧边进程，无残留

### 启动状态

| 状态 | 含义 | 用户看到 |
|------|------|---------|
| `loading` | 侧边进程启动中，等待端口 | 旋转圈 + 进度条动画 |
| `ready` | 侧边进程就绪，WebView 导航到 dsh UI | dsh 完整界面 |
| `error` | 启动失败（dsh 未找到、端口超时等） | 红色错误提示 + 详细信息 |

---

## 项目结构

```
dsh-desktop/
├── .npmrc                    # pnpm 配置（关闭原生构建脚本）
├── pnpm-workspace.yaml       # 工作区定义
├── package.json              # 根调度脚本
├── scripts/
│   ├── install.js            # 安装依赖
│   ├── build.js              # 多平台构建脚本
│   ├── dev.js                # 开发模式入口
│   ├── clean.js              # 清理脚本
│   └── fetch-node.js         # 下载便携 Node.js
├── dsh-app-desktop/
│   ├── package.json          # dsh-app-desktop 子项目
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── index.html            # 启动加载页（SSR 前展示）
│   ├── src/                  # 前端 (React + TypeScript)
│   │   ├── main.tsx
│   │   ├── App.tsx           # 状态感知的启动 UI
│   │   └── index.css
│   └── src-tauri/            # 后端 (Rust + Tauri)
│       ├── Cargo.toml
│       ├── build.rs          # 构建时拷贝 dsh 资源 + 裁剪依赖
│       ├── tauri.conf.json
│       ├── binaries/         # 便携 Node.js (自动下载)
│       ├── resources/
│       │   └── dsh/          # dsh 内核 + 依赖 (自动拷贝)
│       ├── capabilities/     # Tauri 权限配置
│       └── src/
│           ├── main.rs
│           ├── lib.rs        # 应用入口 + 侧边进程编排
│           ├── error.rs      # 错误类型定义
│           ├── paths.rs      # dsh 路径解析（多级 fallback）
│           ├── dsh/          # dsh 生命周期管理
│           │   ├── mod.rs    # SidecarHandle 定义 + 单元测试
│           │   ├── port.rs   # 端口发现（多正则匹配）
│           │   ├── spawn.rs  # 侧边进程启动
│           │   └── shutdown.rs
│           ├── commands/     # Tauri 命令
│           │   ├── mod.rs
│           │   ├── start.rs
│           │   ├── stop.rs
│           │   └── status.rs
│           └── capabilities/ # 原生能力桥接
│               ├── mod.rs
│               ├── file_picker.rs
│               ├── notifications.rs
│               └── opener.rs
```

---

## 构建细节

### 平台映射

| 平台 | Rust 目标三元组 | 脚本参数 |
|------|----------------|---------|
| macOS (ARM64) | `aarch64-apple-darwin` | `macos` |
| macOS (x64) | `x86_64-apple-darwin` | — |
| Windows (x64) | `x86_64-pc-windows-msvc` | `windows` |
| Linux (x64) | `x86_64-unknown-linux-gnu` | `linux` |

### 跨平台构建

在 macOS 上可以交叉编译 Windows 和 Linux 版本：

```bash
pnpm build all
```

### Node.js 便携版

每个目标平台需要对应的 Node.js 可执行文件，`build.js` 会在构建时自动下载：

- 从 [nodejs.org](https://nodejs.org) 下载预编译二进制
- 存放在 `dsh-app-desktop/src-tauri/binaries/`，按 `node-{target-triple}` 命名
- 通过 Tauri `externalBin` 机制打包进应用

### 构建优化

构建过程中 `build.rs` 会自动执行以下优化，减小最终包体积：

1. **dsh 源码拷贝** — 从 `node_modules/@deepseek-ai/dsh` 拷贝到 `dsh-app-desktop/resources/dsh/`
2. **依赖拷贝** — 从 pnpm 虚拟存储拷贝运行所需依赖
3. **依赖裁剪** — 移除 `typescript`、`vite`、`esbuild`、`rollup` 等构建时依赖
4. **平台预编译清理** — 只保留当前平台 `node-pty` 的 prebuild 文件
5. **文档清理** — 移除 `CHANGELOG.md`、`CONTRIBUTING.md` 等无用文件

### 构建产物

| 平台 | 产物格式 | 路径 |
|------|---------|------|
| macOS | `.dmg` / `.app` | `target/<triple>/release/bundle/macos/` |
| Windows | `.msi` / `.exe` | `target/<triple>/release/bundle/msi/` |
| Linux | `.deb` / `.AppImage` | `target/<triple>/release/bundle/deb/` |

---

## 更新 dsh 版本

```bash
# 修改 dsh-app-desktop/package.json 中的版本号
# 然后运行：
pnpm install
```

dsh 的版本声明在 `dsh-app-desktop/package.json` 的 `dependencies` 中：

```json
"@deepseek-ai/dsh": "^0.1.0-rc.5"
```

## 清理

```bash
pnpm clean
```

清理以下内容：

| 路径 | 说明 |
|------|------|
| `node_modules/` | 所有依赖 |
| `dsh-app-desktop/node_modules/` | 工作区依赖 |
| `dsh-app-desktop/dist/` | Vite 前端构建产物 |
| `dsh-app-desktop/src-tauri/target/` | Rust 编译产物 |
| `dsh-app-desktop/src-tauri/gen/` | Tauri 代码生成 |
| `dsh-app-desktop/resources/dsh/` | dsh 资源拷贝 |
| `dsh-app-desktop/src-tauri/binaries/` | 下载的 Node.js |
| `.tmp/` | 临时文件 |

---

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | [Tauri 2](https://v2.tauri.app/) |
| 后端语言 | Rust |
| 前端框架 | React 18 + TypeScript |
| 构建工具 | Vite 5 |
| 包管理 | pnpm |
| AI 引擎 | [deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) (dsh) |
| 原生插件 | shell, notification, dialog |

---

## 许可

MIT — 与 [deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) 相同。