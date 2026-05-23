import { ENDPOINTS } from "./config.js";
import { redactNetworkEntry } from "./redaction.js";
import { recordTrace } from "./state.js";

export function startNetworkCapture({ chromeDevtools, state, transport, render }) {
  chromeDevtools.network.onRequestFinished.addListener((entry) => {
    const trace = redactNetworkEntry(entry);
    recordTrace(state, entry);

    transport.send(ENDPOINTS.traces, {
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
