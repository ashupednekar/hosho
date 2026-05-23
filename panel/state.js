export function createCaptureState() {
  return {
    traces: 0,
    errors: 0,
    warnings: 0,
    failedRequests: 0,
    totalRequestMs: 0,
  };
}

export function recordTrace(state, entry) {
  state.traces += 1;
  state.totalRequestMs += Number(entry.time) || 0;

  const status = Number(entry.response?.status) || 0;
  if (status >= 400) {
    state.failedRequests += 1;
  }
}

export function recordConsoleEvent(state, event) {
  if (event.level === "warn") {
    state.warnings += 1;
  }

  if (event.level === "error") {
    state.errors += 1;
  }
}

export function toMetricsPayload(state, tabId) {
  return {
    type: "runtime.metrics",
    capturedAt: new Date().toISOString(),
    tabId,
    traces: state.traces,
    errors: state.errors,
    warnings: state.warnings,
    failedRequests: state.failedRequests,
    averageRequestMs: state.traces ? Math.round(state.totalRequestMs / state.traces) : 0,
  };
}
