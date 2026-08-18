# DeepSeek Harness Desktop

Tauri 2 桌面应用，封装 [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) Web UI，以原生桌面体验运行 AI 工作流。

## 快速开始

```bash
# 1. 安装依赖 + 初始化环境
pnpm bootstrap

# 2. 开发模式
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

## 项目结构

```
dsh-desktop/
├── .npmrc                    # pnpm 配置
├── pnpm-workspace.yaml       # 工作区定义
├── package.json              # 根调度脚本
├── scripts/
│   ├── install.js            # 安装依赖
│   ├── build.js              # 构建脚本
│   ├── dev.js                # 开发模式
│   ├── clean.js              # 清理脚本
│   └── fetch-node.js         # 下载便携 Node.js
├── apps/
│   └── desktop/
│       ├── package.json      # @dsh/desktop 子项目
│       ├── tsconfig.json
│       ├── vite.config.ts
│       ├── index.html
│       ├── src/              # 前端 (React + TypeScript)
│       │   ├── main.tsx
│       │   ├── App.tsx
│       │   └── index.css
│       └── src-tauri/        # 后端 (Rust + Tauri)
│           ├── Cargo.toml
│           ├── build.rs      # 构建时拷贝 dsh 资源
│           ├── tauri.conf.json
│           ├── binaries/     # 便携 Node.js (.gitignored)
│           ├── resources/
│           │   └── dsh/      # dsh 内核 + 依赖 (.gitignored)
│           ├── capabilities/ # Tauri 权限配置
│           └── src/
│               ├── main.rs
│               ├── lib.rs
│               ├── error.rs
│               ├── paths.rs      # dsh 路径解析
│               ├── dsh/          # dsh 生命周期管理
│               │   ├── mod.rs
│               │   ├── port.rs
│               │   ├── spawn.rs
│               │   └── shutdown.rs
│               ├── commands/     # Tauri 命令
│               │   ├── mod.rs
│               │   ├── start.rs
│               │   ├── stop.rs
│               │   └── status.rs
│               └── capabilities/ # 原生能力桥接
│                   ├── mod.rs
│                   ├── file_picker.rs
│                   ├── notifications.rs
│                   └── opener.rs
```

## 架构

```
┌─────────────────────────────────────────────┐
│  Tauri 桌面应用                               │
│  ┌────────────────┐  ┌────────────────────┐  │
│  │  WebView        │  │  Rust 后端          │  │
│  │  (React SPA)    │◄─┤  ┌──────────────┐  │  │
│  │                 │  │  │ dsh 生命周期  │  │  │
│  │ 加载侧边栏 UI   │  │  │ - 启动/停止   │  │  │
│  │                 │  │  │ - 端口发现    │  │  │
│  └────────────────┘  │  └──────┬───────┘  │  │
│                      │         │          │  │
│  ┌────────────────┐  │  ┌──────┴───────┐  │  │
│  │  原生能力桥接   │  │  │  Node.js     │  │  │
│  │ - 文件选择器    │  │  │  侧边进程    │  │  │
│  │ - 通知         │  │  │  (dsh web)  │  │  │
│  │ - 路径打开     │  │  └─────────────┘  │  │
│  └────────────────┘  │                   │  │
│  ┌────────────────┐  │                   │  │
│  │  Tauri 插件    │  │                   │  │
│  │ - shell        │  │                   │  │
│  │ - notification │  │                   │  │
│  │ - dialog       │  │                   │  │
│  └────────────────┘  │                   │  │
└─────────────────────────────┘

Node.js 以 Tauri externalBin 机制打包进应用，
dsh 内核及其依赖作为资源目录（resources/dsh/）打包。
```

### 工作流程

1. **启动时**：Rust 后端通过 `tauri-plugin-shell` 启动 Node.js 侧边进程，运行 `dsh web --port 0`
2. **端口发现**：从侧边进程 stdout 解析动态分配的端口号
3. **加载 UI**：WebView 导航到 `http://127.0.0.1:<port>` 加载 dsh 网页界面
4. **关闭时**：自动终止侧边进程

## 构建细节

### 平台映射

| 平台 | Rust 目标三元组 | 脚本参数 |
|------|----------------|---------|
| macOS (ARM64) | `aarch64-apple-darwin` | `macos` |
| Windows (x64) | `x86_64-pc-windows-msvc` | `windows` |
| Linux (x64) | `x86_64-unknown-linux-gnu` | `linux` |

### 构建产物

构建完成后，产物位于 `apps/desktop/src-tauri/target/<triple>/release/bundle/`：

- **macOS**: `.dmg` / `.app`
- **Windows**: `.msi` / `.exe`
- **Linux**: `.deb` / `.AppImage`

### 跨平台构建

在 macOS 上可以交叉编译 Windows 和 Linux 版本：

```bash
# 需要安装交叉编译工具链
pnpm build all
```

### Node.js 便携版

每个目标平台需要对应的 Node.js 可执行文件，`build.js` 会在构建时自动下载。下载的二进制文件存放在 `apps/desktop/src-tauri/binaries/` 目录，按 `node-{target-triple}` 命名，通过 Tauri `externalBin` 机制打包进应用。

## 开发

```bash
# 初始化
pnpm bootstrap

# 启动开发服务器（带热重载）
pnpm dev
```

Tauri 开发模式会自动启动 WebView 并连接 dsh 侧边进程。前端代码修改会自动刷新。

## 清理

```bash
pnpm clean
```

清理以下内容：

| 路径 | 说明 |
|------|------|
| `node_modules/` | 所有依赖 |
| `apps/desktop/node_modules/` | 工作区依赖 |
| `apps/desktop/dist/` | Vite 前端构建产物 |
| `apps/desktop/src-tauri/target/` | Rust 编译产物 |
| `apps/desktop/src-tauri/gen/` | Tauri 代码生成 |
| `apps/desktop/src-tauri/resources/dsh/` | dsh 资源拷贝 |
| `apps/desktop/src-tauri/binaries/` | 下载的 Node.js |
| `.tmp/` | 临时文件 |

## 许可

MIT — 与 [deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) 相同。