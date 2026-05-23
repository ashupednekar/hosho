import { API_BASE, ENDPOINTS } from "../config.js";
import { recordTrace } from "../state.js";
import { redactNetworkEntry } from "./redaction.js";

const TELEMETRY_ENDPOINT_URLS = Object.values(ENDPOINTS).map(
  (path) => new URL(`${API_BASE}${path}`),
);

export function startNetworkCapture({ chromeDevtools, state, sendTelemetry, render }) {
  chromeDevtools.network.onRequestFinished.addListener((entry) => {
    if (isTelemetryRequest(entry)) return;

    const trace = redactNetworkEntry(entry);
    recordTrace(state, entry);

    sendTelemetry(ENDPOINTS.traces, {
      type: "network.trace",
      capturedAt: new Date().toISOString(),
      tabId: chromeDevtools.inspectedWindow.tabId,
      trace,
    });

    render("trace", {
      method: trace.request?.method,
      url: trace.request?.url,
      status: trace.response?.status,
      ms: Number(entry.time) || 0,
    });
  });
}

function isTelemetryRequest(entry) {
  const requestUrl = entry.request?.url;
  if (!requestUrl) return false;

  try {
    const url = new URL(requestUrl);
    return TELEMETRY_ENDPOINT_URLS.some(
      (endpointUrl) =>
        url.origin === endpointUrl.origin && url.pathname === endpointUrl.pathname,
    );
  } catch (_) {
    return false;
  }
}
