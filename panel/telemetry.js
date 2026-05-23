import { ENDPOINTS, METRICS_FLUSH_INTERVAL_MS } from "./config.js";
import { toMetricsPayload } from "./state.js";

export function createTransport({ apiBase, render }) {
  return {
    async send(path, payload) {
      try {
        await fetch(`${apiBase}${path}`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        });
      } catch (error) {
        render("send failed", {
          path,
          message: error instanceof Error ? error.message : String(error),
        });
      }
    },
  };
}

export function startMetricsFlush({ chromeDevtools, state, transport }) {
  setInterval(() => {
    transport.send(
      ENDPOINTS.metrics,
      toMetricsPayload(state, chromeDevtools.inspectedWindow.tabId),
    );
  }, METRICS_FLUSH_INTERVAL_MS);
}
