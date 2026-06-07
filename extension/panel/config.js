//export const API_BASE = "https://hosho-ingestor.ashupednekar49.workers.dev";
export const API_BASE = "http://localhost:8787";

export const ENDPOINTS = {
  har: "/ingest/har",
  log: "/ingest/console",
};

export const CONSOLE_BUFFER_NAME = "__hoshoConsoleEvents";
export const CONSOLE_INSTALLED_FLAG = "__hoshoConsoleCaptureInstalled";

export const CONSOLE_DRAIN_INTERVAL_MS = 1000;
