import { API_BASE, ENDPOINTS } from "./config.js";
import { installConsoleCapture, startConsoleDrain } from "./capture/console.js";
import { startNetworkCapture } from "./capture/network.js";

export function startPanel({ chromeDevtools, output }) {
  const render = createRender(output);
  const sendJson = async (path, payload) => {
    try {
      render("sending", { path });
      await fetch(`${API_BASE}${path}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(payload),
      });
      render("sent", { path });
    } catch (error) {
      render("send failed", {
        path,
        message: error instanceof Error ? error.message : String(error),
      });
    }
  };

  installConsoleCapture(chromeDevtools);

  startNetworkCapture({
    chromeDevtools,
    sendJson,
    render,
  });

  startConsoleDrain({
    chromeDevtools,
    sendJson,
    render,
  });

  render("ready", {
    apiBase: API_BASE,
    endpoints: ENDPOINTS,
  });
}

function createRender(output) {
  return function render(kind, detail) {
    output.textContent = JSON.stringify({
      kind,
      detail,
    }, null, 2);
  };
}
