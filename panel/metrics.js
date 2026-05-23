import { ENDPOINTS, METRICS_FLUSH_INTERVAL_MS } from "./config.js";
import { toMetricsPayload } from "./state.js";

export function startMetricsFlush({ chromeDevtools, state, transport }) {
  setInterval(() => {
    transport.send(
      ENDPOINTS.metrics,
      toMetricsPayload(state, chromeDevtools.inspectedWindow.tabId),
    );
  }, METRICS_FLUSH_INTERVAL_MS);
}
