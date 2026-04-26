import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

/**
 * Mock the Tauri APIs and `fetch` so component tests can render
 * without a Tauri runtime or the Express sidecar.
 */
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "get_app_version") return "0.1.0-test";
    if (cmd === "get_sidecar_port") return 0;
    return null;
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
  emit: vi.fn(async () => {}),
}));

// Default fetch mock — tests can override per case.
globalThis.fetch = vi.fn(async () =>
  new Response(JSON.stringify({ message: "Hello, world! (from Node sidecar)" }), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  }),
) as unknown as typeof fetch;
