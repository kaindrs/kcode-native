# KCode Native

A proper, efficient, cross-platform native wrapper for the [Kimi Code](https://kimi-code.com/) web interface. Built with [Tauri](https://tauri.app/) (Rust + the OS native webview) instead of bundling a full browser.

![Icon](assets/icon.svg)

## What it does

KCode Native turns your local Kimi Code web UI into a real desktop app with a Dock icon, system tray, global hotkey, and native notifications.

On launch it will:

1. Check whether `kimi web` (or AXIOM's fork-lifted `acode web`) is already running on port `58627`.
2. If not, start it automatically.
3. Capture the one-time Local URL including the bearer token.
4. Open the IDE, already authenticated, in its own native window.

Because the URL and token are resolved at runtime, nothing secret is baked into the app bundle.

## Features

- **Native webview** — WKWebView on macOS, WebView2 on Windows, WebKitGTK on Linux. Tiny bundle, low memory.
- **Auto-start server** — starts `kimi`/`acode web` if it is not already running.
- **System tray** — show/hide the window or quit from the tray icon.
- **Global shortcut** — `Cmd/Ctrl+Shift+K` to raise KCode from anywhere.
- **Native notifications** — auto-granted so the Kimi web UI can show system notifications.
- **Single instance** — opening the app twice raises the existing window.
- **Anti-flicker injection** — disables smooth scrolling/overscroll behavior that caused chat flicker at the bottom.
- **Cross-platform** — macOS, Windows, Linux builds from one codebase.

## Download

Grab the latest release from the [Releases](https://github.com/kaindrs/kcode-native/releases) page.

## Build from source

### Requirements

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 18+ (only for the Tauri CLI helper scripts)
- `kimi` or `acode` CLI installed (runtime dependency)

### macOS / Linux

```bash
git clone https://github.com/kaindrs/kcode-native.git
cd kcode-native
npm install
npm run build
```

The built app appears under `src-tauri/target/release/bundle/`.

### Windows

```powershell
git clone https://github.com/kaindrs/kcode-native.git
cd kcode-native
npm install
npm run build
```

### Development

```bash
npm run dev
```

This starts the app in development mode. It will still auto-start `kimi web` on port `58627`.

## Configuration

| Variable           | Default  | Description                                              |
|--------------------|----------|----------------------------------------------------------|
| `KCODE_PORT`       | `58627`  | Port `kimi web` / `acode web` listens on                 |
| `KCODE_TIMEOUT`    | `30`     | Seconds to wait for a fresh server to print its URL      |
| `KIMI_CODE_HOME`   | auto     | Override the Kimi Code config directory                  |

## First launch

The app is ad-hoc signed. If macOS shows a security warning, right-click the app and choose **Open**, or run:

```bash
xattr -dr com.apple.quarantine ~/Applications/KCode.app
```

## Project structure

```
kcode-native/
├── assets/              # App icon (PNG + SVG source)
├── src-tauri/           # Rust Tauri project
│   ├── src/main.rs      # App lifecycle, tray, shortcuts
│   ├── src/kcode.rs     # kimi/acode process management
│   ├── assets/          # Injected CSS/JS fixes
│   ├── icons/           # Generated icon set
│   ├── tauri.conf.json
│   └── Cargo.toml
├── package.json
├── README.md
└── LICENSE
```

## License

MIT © kaindrs
