# DeepSeek Harness Desktop

Tauri 2 desktop application wrapping the [deepseek-harness](https://github.com/deepseek-ai/deepseek-harness) web UI.

## Architecture

```
┌─────────────────────────────────────────┐
│  Tauri 2 App (Rust + WebView)          │
│                                         │
│  ┌─ window.rs ────┐   ┌─ capabilities ─┐│
│  │ macOS menu bar  │   │ file picker    ││
│  │ tray icon       │   │ notifications  ││
│  │ window state    │   │ path opener    ││
│  └─────────────────┘   └────────────────┘│
│                                         │
│  WebView loads http://127.0.0.1:<port>  │
└──────────────────┬──────────────────────┘
                   │
              /api/*, /plugins/*, /assets/*
                   │
┌──────────────────▼──────────────────────┐
│  Node.js Sidecar (harness web profile) │
│                                         │
│  cordis.yml bundle: dsh-base + web-app  │
│                                         │
│  Hosts:                                   │
│    - Vite dev server (dev mode)          │
│    - Static dist server (release mode)   │
│    - /api/* gateway (RPC over fetch)     │
│    - /api/events/* (WebSocket SSE)       │
└─────────────────────────────────────────┘
```

**Core principle**: The desktop app is a thin shell. The harness sidecar owns all business logic; Rust owns only native capabilities the browser cannot access.

## Directory Structure

```
deepseek-desktop/
├── src/                     # Rust source
│   ├── main.rs              # App entry: init tracing → run()
│   └── lib.rs               # Tauri builder, sidecar spawn, window lifecycle
│   ├── sidecar.rs           # Sidecar process lifecycle (spawn, port discovery)
│   └── sidecar/
│       └── port_parser.rs   # Regex-based port extraction from harness stdout
│   └── capabilities/        # Native capability handlers
│       ├── mod.rs
│       ├── file_picker.rs   # Tauri dialog → harness hostPickDirectory bridge
│       ├── notifications.rs # System notifications via Tauri
│       └── opener.rs        # Native path opener (open / xdg-open / start)
├── harness-profile/         # Cordis desktop profile (web bundless + patches)
│   ├── package.json
│   ├── cordis.yml
│   └── cordis.patch.yml
├── icons/                   # App icons
├── index.html               # Loading page shown before sidecar connects
├── tauri.conf.json          # Tauri 2 configuration
├── Cargo.toml               # Rust dependencies
├── package.json             # Node dependencies, scripts
├── pnpm-workspace.yaml      # Workspace with local harness during dev
├── build.rs                 # Tauri build script
└── CLAUDE.md                # This file
```

## Dependency on deepseek-harness

This project depends on the [`@deepseek-ai/dsh`](https://www.npmjs.com/package/@deepseek-ai/dsh) npm package (published by DeepSeek AI). The harness runs as an external `dsh web` process.

During local development, `pnpm-workspace.yaml` links to the local `../deepseek-harness` checkout so `node_modules/.bin/dsh` resolves to the source tree. For downstream consumers, the package is resolved from npm.

### Dev mode
```bash
pnpm install    # links local harness via workspace
pnpm dev        # runs `dsh web --port 0` via tsx
```

### Release mode
```bash
pnpm install    # installs @deepseek-ai/dsh from npm
pnpm build      # bundles into native executable
```

## Key Design Decisions

### Sidecar port selection
The harness `web` profile picks a random port when `--port 0` is passed. The Rust sidecar module captures this port via stdout regex parsing (`listening on 127.0.0.1:(\d+)`) and passes it to the WebView before navigation.

### Native capability bridging
The harness web frontend calls `hostOpenPath`, `hostPickDirectory`, `hostListDirectory` via the API proxy. These are handled by the Node.js sidecar's existing implementations (`native-path-opener.ts`, `directory-picker-*`).

For a proper desktop experience, the Tauri app overrides:
- **File picker**: Use Tauri's native `FileDialogBuilder` instead of the browser-based browse picker
- **Notifications**: Use Tauri's `Notification` API for session events
- **Path opener**: Use Tauri's `open` command for better cross-platform support

These overrides are implemented as Cordis plugins that register in the sidecar's profile, injected via the desktop app's `cordis.patch.yml`.

### Boot manifest injection
The harness's `dsh-client-modules` package injects `window.__DSH_BOOT__` into the served HTML via `webServer.tapIndex()`. This works automatically in both dev and release modes since the sidecar handles HTML serving. No manual intervention needed.

### Privacy and trust fence
The harness enforces a loopback-only trust fence by default (`--host 127.0.0.1`). The desktop sidecar binds to this same address, so no `--trusted-host` configuration is required. The Tauri WebView, running within the same process as the sidecar, naturally satisfies this constraint.

## Building from Source

### Prerequisites
- Rust toolchain (cargo ≥ 1.75)
- Node.js ≥ 22.19 or ≥ 24
- pnpm ≥ 11.7
- Tauri prerequisites: https://tauri.app/start/prerequisites/

### First-time setup
```bash
cd deepseek-desktop
pnpm install
```

### Run in dev mode
```bash
pnpm dev
```

### Build release
```bash
pnpm build
```

### Lint and typecheck
```bash
pnpm check  # runs clippy + rustfmt checks via cargo
```

## Code Conventions

- **Rust**: Follow [rust-lang/naming-conventions](https://rust-lang.github.io/api-guidelines/) and the [clippy](https://doc.rust-lang.org/clippy/) style guide
- **Error handling**: Use `anyhow` for application errors, `thiserror` for error types
- **Logging**: Use `tracing` for structured logging
- **Async**: Use `tokio` runtime; prefer `async/await` over `.await` chains
- **Naming**: snake_case for functions/variables, PascalCase for types, UPPER_SNAKE for constants

## Testing

```bash
cargo test          # unit tests
cargo clippy        # lints
cargo fmt --check   # formatting check
```

## License

MIT — same as deepseek-harness.
