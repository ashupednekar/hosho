import { API_BASE, ENDPOINTS } from "../config.js";
import { sanitizedUrl } from "./url.js";

const INGEST_ENDPOINT_URLS = Object.values(ENDPOINTS).map(
  (path) => new URL(`${API_BASE}${path}`),
);

export function startNetworkCapture({ chromeDevtools, sendJson, render }) {
  chromeDevtools.network.onRequestFinished.addListener((entry) => {
    if (isIngestRequest(entry)) return;

    sendJson(ENDPOINTS.har, minimalNetworkEvent(entry));

    render("trace", {
      method: entry.request?.method,
      url: sanitizedUrl(entry.request?.url),
      status: entry.response?.status,
      ms: Number(entry.time) || 0,
    });
  });
}

function minimalNetworkEvent(entry) {
  return {
    request: minimalRequest(entry.request),
    response: minimalResponse(entry.response),
    timing: minimalTiming(entry),
    trace: minimalTrace(entry),
  };
}

function minimalRequest(request = {}) {
  return {
    method: request.method,
    url: sanitizedUrl(request.url),
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
  };
}

function minimalTiming(entry) {
  const timings = entry.timings || {};
  return {
    startedAt: entry.startedDateTime,
    durationMs: nonNegativeNumber(entry.time),
    blockedMs: nonNegativeNumber(timings.blocked),
    dnsMs: nonNegativeNumber(timings.dns),
    connectMs: nonNegativeNumber(timings.connect),
    sslMs: nonNegativeNumber(timings.ssl),
    sendMs: nonNegativeNumber(timings.send),
    waitMs: nonNegativeNumber(timings.wait),
    receiveMs: nonNegativeNumber(timings.receive),
  };
}

function minimalTrace(entry) {
  const frame = entry._initiator?.stack?.callFrames?.[0];
  return {
    traceparent: headerValue(entry.request?.headers, "traceparent"),
    trigger: frame
      ? {
          functionName: frame.functionName,
          url: sanitizedUrl(frame.url),
          lineNumber: frame.lineNumber,
          columnNumber: frame.columnNumber,
        }
      : undefined,
  };
}

function headerValue(headers = [], name) {
  return headers.find((header) => header.name?.toLowerCase() === name)?.value;
}

function nonNegativeNumber(value) {
  const number = Number(value);
  return Number.isFinite(number) && number >= 0 ? number : undefined;
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
