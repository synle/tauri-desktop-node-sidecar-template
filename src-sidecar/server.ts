import express, { type Request, type Response } from "express";
import cors from "cors";
import { createServer } from "node:http";

/**
 * Tauri sidecar entry point.
 *
 * Listens on a dynamic port (port 0) when `SIDECAR_PORT=0` is passed by the
 * Tauri shell, otherwise honors `SIDECAR_PORT`. Once bound, prints
 * `__SIDECAR_PORT__=<n>` so the Rust shell can parse the actual port from
 * stdout. Reads stdin so it can detect when the parent (Tauri) exits and
 * shut itself down.
 */
const app = express();
app.use(cors({ origin: "*" }));
app.use(express.json({ limit: "10mb" }));

/** Health check. */
app.get("/api/health", (_req: Request, res: Response) => {
  res.json({ ok: true, ts: Date.now() });
});

/** Sample echo endpoint. */
app.post("/api/echo", (req: Request, res: Response) => {
  res.json({ received: req.body ?? null });
});

/** Sample command — returns a greeting. */
app.get("/api/greet", (req: Request, res: Response) => {
  const name = String(req.query.name ?? "world");
  res.json({ message: `Hello, ${name}! (from Node sidecar)` });
});

const requestedPort = Number(process.env.SIDECAR_PORT ?? 3001);
const server = createServer(app);

server.listen(requestedPort, "127.0.0.1", () => {
  const addr = server.address();
  const port = typeof addr === "object" && addr ? addr.port : requestedPort;
  // The Tauri shell parses this exact line to learn the port.
  console.log(`__SIDECAR_PORT__=${port}`);
});

// Parent-death detection: when Tauri exits, our stdin pipe closes -> EOF -> exit.
process.stdin.resume();
process.stdin.on("end", () => {
  // eslint-disable-next-line no-console
  console.log("sidecar: parent stdin closed, exiting");
  process.exit(0);
});
process.on("SIGINT", () => process.exit(0));
process.on("SIGTERM", () => process.exit(0));
