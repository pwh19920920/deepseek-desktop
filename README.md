# DeepSeek Harness Desktop

Tauri 2 desktop application wrapping the [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) web UI.

## Prerequisites

- Rust toolchain (≥ 1.77)
- Node.js ≥ 22.19 or ≥ 24
- pnpm ≥ 11.7
- [Tauri prerequisites](https://tauri.app/start/prerequisites/)

## Development

```bash
pnpm install
pnpm dev
```

## Building

```bash
pnpm build
```

## Architecture

The desktop app spawns the harness as a Node.js sidecar process (`dsh web --profile web --port 0`), discovers the assigned port from stdout, and loads it in a Tauri WebView. Native capabilities (file picker, notifications, path opener) are bridged via Tauri Commands.

See [CLAUDE.md](./CLAUDE.md) for full architecture details.

## License

MIT — same as deepseek-harness.
