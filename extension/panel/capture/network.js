import { API_BASE, ENDPOINTS } from "../config.js";

const INGEST_ENDPOINT_URLS = Object.values(ENDPOINTS).map(
  (path) => new URL(`${API_BASE}${path}`),
);

export function startNetworkCapture({ chromeDevtools, sendJson, render }) {
  chromeDevtools.network.onRequestFinished.addListener((entry) => {
    if (isIngestRequest(entry)) return;

    sendJson(ENDPOINTS.har, entry);

    render("trace", {
      method: entry.request?.method,
      url: entry.request?.url,
      status: entry.response?.status,
      ms: Number(entry.time) || 0,
    });
  });
}

function isIngestRequest(entry) {
  const requestUrl = entry.request?.url;
  if (!requestUrl) return false;

  try {
    const url = new URL(requestUrl);
    return INGEST_ENDPOINT_URLS.some(
      (endpointUrl) =>
        url.origin === endpointUrl.origin && url.pathname === endpointUrl.pathname,
    );
  } catch (_) {
    return false;
  }
}
