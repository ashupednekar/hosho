import { API_BASE, ENDPOINTS } from "../config.js";

const INGEST_ENDPOINT_URLS = Object.values(ENDPOINTS).map(
  (path) => new URL(`${API_BASE}${path}`),
);

export function startNetworkCapture({ chromeDevtools, sendJson, render }) {
  chromeDevtools.network.onRequestFinished.addListener((entry) => {
    if (isIngestRequest(entry)) return;

    sendJson(ENDPOINTS.har, {
      schema: "hosho.har.entry.v1",
      capture: {
        source: "chrome.devtools.network.onRequestFinished",
        tabId: chromeDevtools.inspectedWindow.tabId,
        pageUrl: entry.request?.url,
        capturedAt: new Date().toISOString(),
      },
      entry: minimalHarEntry(entry),
    });

    render("trace", {
      method: entry.request?.method,
      url: entry.request?.url,
      status: entry.response?.status,
      ms: Number(entry.time) || 0,
    });
  });
}

function minimalHarEntry(entry) {
  return {
    startedDateTime: entry.startedDateTime,
    time: entry.time,
    request: minimalRequest(entry.request),
    response: minimalResponse(entry.response),
    timings: entry.timings,
    _requestId: entry._requestId,
    _resourceType: entry._resourceType,
    _priority: entry._priority,
    _connectionId: entry._connectionId,
    _initiator: minimalInitiator(entry._initiator),
  };
}

function minimalRequest(request = {}) {
  return {
    method: request.method,
    url: request.url,
    httpVersion: request.httpVersion,
    headers: request.headers,
    headersSize: request.headersSize,
    bodySize: request.bodySize,
  };
}

function minimalResponse(response = {}) {
  return {
    status: response.status,
    statusText: response.statusText,
    httpVersion: response.httpVersion,
    headers: response.headers,
    headersSize: response.headersSize,
    bodySize: response.bodySize,
    content: response.content ? { mimeType: response.content.mimeType } : undefined,
    _fromDiskCache: response._fromDiskCache,
    _fromMemoryCache: response._fromMemoryCache,
  };
}

function minimalInitiator(initiator) {
  if (!initiator) return undefined;
  const frame = initiator.stack?.callFrames?.[0];
  return {
    type: initiator.type,
    stack: frame ? { callFrames: [frame] } : undefined,
  };
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
