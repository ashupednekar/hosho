export const API_BASE = "https://your-api.example.com";

export const ENDPOINTS = {
  traces: "/hosho/traces",
  errors: "/hosho/errors",
  metrics: "/hosho/metrics",
};

export const CONSOLE_BUFFER_NAME = "__hoshoConsoleEvents";
export const CONSOLE_INSTALLED_FLAG = "__hoshoConsoleCaptureInstalled";

export const CONSOLE_DRAIN_INTERVAL_MS = 1000;
export const METRICS_FLUSH_INTERVAL_MS = 10000;
