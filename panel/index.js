import { API_BASE } from "./config.js";
import { installConsoleCapture, startConsoleDrain } from "./capture/console.js";
import { startNetworkCapture } from "./capture/network.js";
import { createCaptureState } from "./state.js";
import { createTransport, startMetricsFlush } from "./telemetry.js";
import { createRenderer } from "./ui.js";

const output = document.getElementById("output");
const state = createCaptureState();
const render = createRenderer(output, state);
const transport = createTransport({ apiBase: API_BASE, render });

installConsoleCapture(chrome.devtools);

startNetworkCapture({
  chromeDevtools: chrome.devtools,
  state,
  transport,
  render,
});

startConsoleDrain({
  chromeDevtools: chrome.devtools,
  state,
  transport,
  render,
});

startMetricsFlush({
  chromeDevtools: chrome.devtools,
  state,
  transport,
});

render("ready", {
  traces: "/api/traces",
  errors: "/api/errors",
  metrics: "/api/metrics",
});
