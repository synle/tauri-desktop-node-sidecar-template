# Tauri Desktop Node Sidecar Template

Skeleton template for a cross-platform desktop app using **Tauri v2** + **Node.js/Express sidecar** + **React 19** (TypeScript) + **MUI v9** + **Vite 6** + **Vitest 4**. The Tauri Rust shell spawns a Node.js child process running an Express server on a dynamic port; the React frontend talks to it over `http://127.0.0.1:<port>/api/...`.

Use this template when your backend logic is most natural in JavaScript/TypeScript (e.g. database drivers, npm-only libraries, an existing Node API).

Two starter pages — **Home** (calls `GET /api/greet`) and **Settings** — wired up via React Router (HashRouter).

## Requirements

| Tool          | Version | Notes                                                                 |
| ------------- | ------- | --------------------------------------------------------------------- |
| Node.js       | 20+     | Use `fnm` / `nvm` to pin                                              |
| npm           | 10+     | Ships with Node                                                       |
| Rust          | stable  | `rustup default stable`                                               |
| Tauri prereqs | —       | See [tauri.app prerequisites](https://tauri.app/start/prerequisites/) |

Platform-specific extras:

- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Windows**: Microsoft C++ Build Tools, WebView2 (preinstalled on Win11)
- **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libxdo-dev libssl-dev`

> The packaged `.app` / `.exe` requires **Node.js to be installed system-wide** at runtime — GUI apps can't see version managers (fnm/nvm/volta), so install Node via `brew install node`, the official `.pkg`, or place a `node` binary in `/usr/local/bin`. The Rust shell probes common locations (see `find_system_node()` in `src-tauri/src/lib.rs`). If you'd rather bundle a Node binary, copy display-dj's `scripts/download-node.js` and place the result in `src-tauri/resources/`.

## Getting started

```bash
git clone <this-repo>
cd tauri-desktop-node-sidecar-template
npm install            # install JS dependencies
npx tauri dev          # run the desktop app + sidecar in dev mode
```

In dev mode, `npm run dev` runs Vite + the Express sidecar (rebuilt on save by Vite, restarted by nodemon) on port `3001`. The Vite dev server proxies `/api/*` to it. The Tauri Rust shell does **not** spawn the sidecar in `tauri dev` (`#[cfg(debug_assertions)]`) — it only spawns it in production builds.

Useful scripts:

```bash
npm run dev:web        # Vite frontend only (browser at http://localhost:1420)
npm run dev:sidecar    # Express sidecar only (port 3001), with hot rebuild
npm run build          # Production frontend build
npm run build:sidecar  # Bundle src-sidecar/server.ts -> src-tauri/resources/server.cjs
npm run build:tauri    # build + build:sidecar (used as Tauri beforeBuildCommand)
npm test               # Vitest run
npm run typecheck      # tsc --noEmit
npm run tauri:build    # Production desktop build (.dmg/.exe/.deb/.AppImage)
cd src-tauri && cargo test  # Rust tests
```

## Project layout

```
.
├── src/                       # React frontend
│   ├── components/NavBar.tsx
│   ├── pages/                 # HomePage (calls sidecar), SettingsPage
│   ├── test/setup.ts          # Vitest setup (mocks Tauri + fetch)
│   ├── App.tsx
│   └── main.tsx
├── src-sidecar/
│   └── server.ts              # Express app — entry point for the sidecar
├── src-tauri/                 # Tauri Rust shell
│   ├── src/lib.rs             # Spawns sidecar, exposes get_sidecar_port()
│   ├── resources/             # Bundled .cjs lives here in prod
│   └── tauri.conf.json
├── vite.frontend.config.ts    # Vite config for the React UI
├── vite.sidecar.config.ts     # Vite SSR config that bundles the sidecar
└── .github/workflows/         # build, release-official, release-beta
```

## How the sidecar wires up

1. **Build**: `vite.sidecar.config.ts` bundles `src-sidecar/server.ts` and all its npm deps into a single `src-tauri/resources/server.cjs`. `tauri.conf.json` ships everything in `resources/` with the app.
2. **Start**: In production builds, `lib.rs::spawn_sidecar()` calls `node resources/server.cjs` with `SIDECAR_PORT=0` and `stdin` piped. The sidecar binds a random port and prints `__SIDECAR_PORT__=<n>` to stdout; the Rust side parses it.
3. **Discover**: The frontend calls `invoke('get_sidecar_port')` to learn the port, then `fetch(\`http://127.0.0.1:${port}/api/...\`)`.
4. **Shutdown**: When Tauri exits, the stdin pipe closes; the sidecar sees EOF on `process.stdin` and calls `process.exit(0)`. Tauri also `child.kill()`s as a backup on `RunEvent::Exit`.

## Versioning & release

The version lives in **`src-tauri/tauri.conf.json` → `version`**. `build.rs` exposes it as `APP_VERSION`. Dev builds append `[DEV]`; CI release builds set `TAURI_RELEASE=true`.

- **Build CI** (`.github/workflows/build.yml`) — runs on every push/PR to `main`. Tests + builds on macOS (ARM + Intel), Windows, Linux.
- **Official release** (`.github/workflows/release-official.yml`) — `v*` tag or `workflow_dispatch`.
- **Beta release** (`.github/workflows/release-beta.yml`) — manual `workflow_dispatch` only.

## What to change after cloning

1. Rename in `package.json` (`name`, `description`).
2. Rename in `src-tauri/Cargo.toml` (`[package].name`, `[lib].name`) and `src-tauri/src/main.rs` (`app_lib::run()`).
3. Update `src-tauri/tauri.conf.json` (`productName`, `identifier`, `windows[].title`).
4. Replace icons in `src-tauri/icons/` (`npx tauri icon path/to/icon.png`).
5. Update `.github/workflows/release-*.yml` `project_name`.
6. Replace the sample `/api/greet` endpoint in `src-sidecar/server.ts`.

## License

MIT — add a `LICENSE` file if you publish.
