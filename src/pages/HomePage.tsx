import { useEffect, useState } from "react";
import { Box, Button, Card, CardContent, Stack, Typography } from "@mui/material";
import { invoke } from "@tauri-apps/api/core";

/** Resolves the base URL for the sidecar API (Tauri prod -> Rust port; dev -> Vite proxy). */
async function getApiBase(): Promise<string> {
  try {
    const port = await invoke<number>("get_sidecar_port");
    if (port && port > 0) return `http://127.0.0.1:${port}`;
  } catch {
    // not running under Tauri (e.g. browser/tests) — fall through.
  }
  return ""; // relative — Vite proxy handles `/api/*`
}

/** Home page — fetches version + greeting from the Node sidecar over HTTP. */
export default function HomePage() {
  const [version, setVersion] = useState<string>("");
  const [greeting, setGreeting] = useState<string>("");

  useEffect(() => {
    invoke<string>("get_app_version")
      .then(setVersion)
      .catch(() => setVersion("(running outside Tauri)"));
  }, []);

  const handleGreet = async () => {
    try {
      const base = await getApiBase();
      const r = await fetch(`${base}/api/greet?name=world`);
      const data = (await r.json()) as { message: string };
      setGreeting(data.message);
    } catch (e) {
      setGreeting(`Error: ${e}`);
    }
  };

  return (
    <Stack spacing={3}>
      <Typography variant="h4">Home</Typography>
      <Card>
        <CardContent>
          <Typography variant="subtitle2" color="text.secondary">
            App version
          </Typography>
          <Typography variant="h6">{version || "loading..."}</Typography>
        </CardContent>
      </Card>
      <Box>
        <Button variant="contained" onClick={handleGreet}>
          Call sidecar /api/greet
        </Button>
        {greeting && (
          <Typography sx={{ mt: 2 }} variant="body1">
            {greeting}
          </Typography>
        )}
      </Box>
    </Stack>
  );
}
