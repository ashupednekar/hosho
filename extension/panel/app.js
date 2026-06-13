import { API_BASE, ENDPOINTS } from "./config.js";
import { installConsoleCapture, startConsoleDrain } from "./capture/console.js";
import { startNetworkCapture } from "./capture/network.js";

export function startPanel({ chromeDevtools, output, errorList, status }) {
  const render = createRender({ output, errorList, status });
  const sendJson = async (path, payload) => {
    try {
      render("sending", { path });
      const response = await fetch(`${API_BASE}${path}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        render("send failed", {
          path,
          ...(await responseError(response)),
        });
        return;
      }
      render("sent", { path });
    } catch (error) {
      render("send failed", {
        path,
        code: "ERR-PANEL-SEND",
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

function createRender({ output, errorList, status }) {
  const errors = new Map();

  return function render(kind, detail) {
    if (status) status.textContent = kind;
    if (kind === "send failed") {
      const code = detail.code || codeFromMessage(detail.message) || "ERR-PANEL-UNKNOWN";
      const current = errors.get(code);
      errors.set(code, {
        code,
        count: current ? current.count + 1 : 1,
        message: detail.detail || detail.message || "Unknown error",
        path: detail.path,
        status: detail.status,
      });
      renderErrors(errorList, errors);
    }

    output.textContent = JSON.stringify({
      kind,
      detail,
    }, null, 2);
  };
}

async function responseError(response) {
  const text = await response.text();
  let body = {};
  try {
    body = text ? JSON.parse(text) : {};
  } catch (_) {
    body = {};
  }

  return {
    status: response.status,
    code: body.code || body.err_code || `HTTP-${response.status}`,
    detail: body.detail || body.message || text || response.statusText,
  };
}

function renderErrors(errorList, errors) {
  if (!errorList) return;
  errorList.replaceChildren();

  if (errors.size === 0) {
    const item = document.createElement("li");
    item.className = "empty";
    item.textContent = "No errors";
    errorList.append(item);
    return;
  }

  for (const error of errors.values()) {
    const item = document.createElement("li");
    item.className = "error-row";

    const code = document.createElement("strong");
    code.textContent = error.code;

    const count = document.createElement("span");
    count.className = "count";
    count.textContent = `x${error.count}`;

    const message = document.createElement("span");
    message.className = "message";
    message.textContent = error.message;

    item.append(code, count, message);
    errorList.append(item);
  }
}

function codeFromMessage(message = "") {
  return String(message).match(/\b[A-Z]+-[A-Z0-9-]+\b/)?.[0];
}
