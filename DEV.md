# tauri-desktop-node-sidecar-template

Skeleton Tauri v2 desktop app template with a bundled Node.js / Express sidecar process. Frontend is React 19 + MUI v9 + TypeScript, built with Vite; the sidecar is a CommonJS Express server packaged as a Tauri resource.

## Quick Start

Install dependencies:

```bash
npm ci || npm install --no-fund --prefer-offline
```

Run the desktop app in dev mode (starts Vite, the sidecar with nodemon, and the Tauri shell):

```bash
npm run tauri:dev
```

Build a production bundle (dmg / nsis / deb / appimage):

```bash
npm run tauri:build
```
