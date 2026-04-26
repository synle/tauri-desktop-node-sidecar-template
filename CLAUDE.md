# CLAUDE.md

Guidance for Claude Code when working in this repository.

## Project Overview

A **skeleton template** for cross-platform desktop apps built with **Tauri v2** + **Node.js/Express sidecar** + **React 19** (TypeScript) + **MUI v9** + **Vite 6**. The Tauri Rust shell spawns a Node.js child process running an Express server on a dynamic port; the React frontend talks to it over HTTP.

Sister templates (in adjacent directories):

- `tauri-desktop-raw-template` — plain Tauri (no sidecar)
- `tauri-desktop-shell-sidecar-template` — Tauri + shell binary sidecar

## Build commands

```bash
npm install                # JS dependencies
npx tauri dev              # Full app + sidecar in dev mode
npm run dev:web            # Vite frontend only (browser mode)
npm run dev:sidecar        # Express sidecar only (port 3001), with hot rebuild
npm run build              # Production frontend build
npm run build:sidecar      # Bundle src-sidecar/server.ts -> src-tauri/resources/server.cjs
npm run build:tauri        # build + build:sidecar (Tauri's beforeBuildCommand)
npx tauri build            # Production desktop build
npm test                   # Vitest (run once)
cd src-tauri && cargo test # Rust tests
```

## Architecture

Three layers:

- **`src/` (React + TS)** — UI built with MUI v9. Routes via React Router (`HashRouter`). Talks to the sidecar over `fetch()`. The frontend resolves the sidecar URL by calling `invoke('get_sidecar_port')`. In dev mode the call returns `0` and the frontend uses relative URLs so the Vite proxy forwards `/api/*` to port `3001`.
- **`src-sidecar/server.ts` (Node + Express)** — listens on a dynamic port (pass `SIDECAR_PORT=0`). Prints `__SIDECAR_PORT__=<n>` on stdout so the Rust shell can parse it. Reads stdin so it can detect parent (Tauri) death and exit cleanly.
- **`src-tauri/` (Rust)** — Tauri v2 shell. In production builds, `lib.rs::spawn_sidecar()` runs `node resources/server.cjs`, parses the port from stdout, and exposes it to the frontend via the `get_sidecar_port` Tauri command. On exit, drops the sidecar's stdin and `child.kill()`s as a backup.

### Production sidecar lifecycle

1. **Build**: `vite.sidecar.config.ts` runs in SSR mode with `noExternal: true` and produces a single `src-tauri/resources/server.cjs` (all npm deps inlined; only Node built-ins external).
2. **Spawn**: `find_system_node()` probes fnm/nvm/volta/mise/n/asdf/nodenv/Homebrew paths since GUI apps don't inherit shell PATH on macOS/Linux. Falls back to plain `node`.
3. **Port handshake**: Sidecar reports `__SIDECAR_PORT__=<n>` on stdout; Rust parses with a 15 s timeout.
4. **Shutdown**: stdin EOF triggers `process.exit(0)` in the sidecar; Rust force-kills after 3 s if the child hasn't exited.

### Dev mode

`tauri dev` runs `npm run dev` (Vite + sidecar via `concurrently`) and the Rust shell skips spawning the sidecar (`#[cfg(debug_assertions)]`). The sidecar listens on `3001`; Vite proxies `/api/*` to it. `get_sidecar_port` returns `0` to signal "use relative URLs".

## Versioning

The single source of truth is **`src-tauri/tauri.conf.json` → `version`**. `build.rs` exposes it as `APP_VERSION`. Dev builds append `[DEV]`; CI release builds set `TAURI_RELEASE=true` for clean version strings.

## Conventions

- All API responses use `camelCase` JSON.
- Tauri commands are `snake_case` in Rust; the frontend calls them with `snake_case` strings.
- Frontend never imports anything from `src-sidecar/` — that code only runs in Node.
- Always add tests for new code: components get `*.test.tsx` (Vitest + Testing Library), Rust modules get `#[cfg(test)] mod tests`.

## CI / Release Workflows

- **`build.yml`** — runs on every push/PR to `main`, runs `npm test` and `cargo test` then builds the Tauri bundle on macOS (ARM + Intel), Windows, Linux. Posts a PR comment with artifact download links.
- **`release-official.yml`** — `v*` tag pushes or manual `workflow_dispatch`. Uses `synle/workflows/actions/release/{begin,end}-release` for the unified flow.
- **`release-beta.yml`** — manual `workflow_dispatch` only. Builds a draft prerelease.

Use the `/release-official` and `/release-beta` slash commands to trigger interactively.

## What to update when adapting this template

1. `package.json` → `name`, `description`
2. `src-tauri/Cargo.toml` → `[package].name`, `[lib].name` (and update `src-tauri/src/main.rs` to match)
3. `src-tauri/tauri.conf.json` → `productName`, `identifier`, `windows[].title`, `version`
4. `src-tauri/icons/` → replace (use `npx tauri icon path/to/icon.png`)
5. `.github/workflows/release-*.yml` → `project_name` strings
6. `src-sidecar/server.ts` → replace the sample endpoints with your own routes
7. `index.html` `<title>`

## GitHub Raw File URLs

Always use the `?raw=1` blob URL format: `https://github.com/{owner}/{repo}/blob/head/{path}?raw=1`.

Do NOT use `api.github.com/repos/.../contents/` or `raw.githubusercontent.com`.

## Git / PR Merge Policy

- Always use **squash and merge** for PRs.
- **Always rebase before pushing** (`git pull --rebase` before `git push`).
