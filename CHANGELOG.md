# Changelog

## v0.1.0 (2026-08-19)

### 中文

**DeepSeek Harness Desktop** 首个公开发布版本 🎉

基于 Tauri 2 将 deepseek-harness AI 工作流引擎打包为原生桌面应用。

#### 功能特性

- **原生轻量** — 基于 Tauri 2，使用系统原生 WebView，不捆绑 Chromium
- **跨平台** — 支持 macOS (ARM64/x64)、Windows (x64)、Linux (x64/ARM64)
- **自包含** — 构建时自动下载便携 Node.js，拷贝并裁剪 dsh 依赖，开箱即用
- **插件市场** — 内置 dsh-market 插件市场，支持 1,250+ 社区插件的一键安装、更新管理、热开关、备份恢复
- **构建优化** — 自动移除构建时依赖，极致减小包体积
- **原生 OS 集成** — 系统通知、文件对话框、默认程序打开文件
- **优雅启动** — 加载动画，实时推送 loading → ready / error 状态
- **干净关闭** — 窗口关闭时自动终止 dsh 侧边进程，无残留

---

### English

The first public release of **DeepSeek Harness Desktop** 🎉

A Tauri 2 desktop shell for the deepseek-harness AI workflow engine.

#### Features

- **Lightweight & Native** — Built with Tauri 2, uses system native WebView, no Chromium bundled
- **Cross-Platform** — macOS (ARM64/x64), Windows (x64), Linux (x64/ARM64)
- **Self-Contained** — Auto-downloads portable Node.js at build time, copies and prunes dsh deps, ready out of the box
- **Plugin Marketplace** — Built-in dsh-market with 1,250+ community plugins: one-click install, updates, hot-switching, backup & restore
- **Build Optimization** — Auto-removes build-time deps, minimizes package size
- **Native OS Integration** — Notifications, file dialogs, default program opener
- **Elegant Startup** — Loading animation with real-time status push (loading → ready / error)
- **Clean Shutdown** — Auto-terminates dsh sidecar on window close, no residual processes