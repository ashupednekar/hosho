import {
  API_BASE,
  ENDPOINTS,
  METRICS_FLUSH_INTERVAL_MS,
} from "./config.js";
import { installConsoleCapture, startConsoleDrain } from "./capture/console.js";
import { startNetworkCapture } from "./capture/network.js";
import { createCaptureState, toMetricsPayload } from "./state.js";

export function startPanel({ chromeDevtools, output }) {
  const state = createCaptureState();
  const render = createRender(output, state);
  const sendTelemetry = (path, payload) => postTelemetry(path, payload, render);

  installConsoleCapture(chromeDevtools);

  startNetworkCapture({
    chromeDevtools,
    state,
    sendTelemetry,
    render,
  });

  startConsoleDrain({
    chromeDevtools,
    state,
    sendTelemetry,
    render,
  });

  startMetricsFlush({
    chromeDevtools,
    state,
    sendTelemetry,
  });

  render("ready", ENDPOINTS);
}

function createRender(output, state) {
  return function render(kind, detail) {
    output.textContent = JSON.stringify({
      kind,
      state,
      detail,
    }, null, 2);
  };
}

async function postTelemetry(path, payload, render) {
  try {
    await fetch(`${API_BASE}${path}`, {
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
}

function startMetricsFlush({ chromeDevtools, state, sendTelemetry }) {
  setInterval(() => {
    sendTelemetry(
      ENDPOINTS.metrics,
      toMetricsPayload(state, chromeDevtools.inspectedWindow.tabId),
    );
  }, METRICS_FLUSH_INTERVAL_MS);
}
