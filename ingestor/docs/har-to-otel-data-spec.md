# HAR to OpenTelemetry data spec

Status: draft v1

This document defines how Hosho should ingest Chrome DevTools HAR entries and emit
OpenTelemetry trace data. Chrome DevTools `network.onRequestFinished` sends one
HAR entry per completed request, not a whole HAR log. The ingestor should accept
both one-entry and batch shapes, normalize them to one internal record per
network request, then export OTLP traces.

## External references

- Chrome DevTools Network API: `onRequestFinished` provides a HAR entry, while
  `getHAR()` provides the full HAR log.
  https://developer.chrome.com/docs/extensions/reference/api/devtools/network
- OpenTelemetry HTTP client semantic conventions:
  https://opentelemetry.io/docs/specs/semconv/http/http-spans/
- OpenTelemetry trace API data model:
  https://opentelemetry.io/docs/specs/otel/trace/api/
- OTLP JSON protobuf encoding:
  https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding
- W3C Trace Context:
  https://www.w3.org/TR/trace-context/

## Ingest envelope

The current extension posts the raw DevTools HAR entry directly to
`POST /ingest/har`. Keep accepting that as legacy input. New clients should send
this envelope:

```json
{
  "schema": "hosho.har.entry.v1",
  "capture": {
    "source": "chrome.devtools.network.onRequestFinished",
    "sessionId": "018f6fd6-4a12-7a4a-b2bc-1ff0fef32b0c",
    "pageId": "tab-125:nav-4",
    "tabId": 125,
    "pageUrl": "https://github.com/org/repo/pull/1",
    "extensionVersion": "0.1.0",
    "capturedAt": "2026-06-08T10:15:30.123Z"
  },
  "redaction": {
    "policy": "default",
    "bodies": "none",
    "requestHeaders": ["authorization", "cookie", "proxy-authorization"],
    "responseHeaders": ["set-cookie"]
  },
  "entry": {}
}
```

For a full HAR export, use the same top-level metadata and send entries as a
batch:

```json
{
  "schema": "hosho.har.batch.v1",
  "capture": {},
  "redaction": {},
  "entries": []
}
```

Server-side normalization rules:

- If the body has `schema = hosho.har.entry.v1`, read `entry`.
- If the body has `schema = hosho.har.batch.v1`, read `entries`.
- If the body looks like a HAR entry and has `request`, `response`, and
  `startedDateTime`, wrap it as `hosho.har.entry.v1`.
- If the body looks like a HAR log and has `log.entries`, wrap each entry using
  `hosho.har.batch.v1`.

## Normalized request record

Each HAR entry becomes one `hosho.network.request.v1` record. This record is the
stable contract between ingestion, storage, and OTel export.

```json
{
  "schema": "hosho.network.request.v1",
  "capture": {
    "source": "chrome.devtools.network.onRequestFinished",
    "sessionId": "018f6fd6-4a12-7a4a-b2bc-1ff0fef32b0c",
    "pageId": "tab-125:nav-4",
    "tabId": 125,
    "pageUrl": "https://github.com/org/repo/pull/1"
  },
  "identity": {
    "requestId": "125636",
    "entryHash": "sha256:...",
    "dedupeKey": "sha256:..."
  },
  "request": {
    "method": "GET",
    "url": "https://github.com/org/repo/pull/1/files?short_path=abc",
    "urlSanitized": "https://github.com/org/repo/pull/1/files?short_path=REDACTED",
    "scheme": "https",
    "host": "github.com",
    "port": 443,
    "httpVersion": "HTTP/2",
    "headers": {
      "traceparent": ["00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"],
      "user-agent": ["Mozilla/5.0 ..."]
    },
    "headersSize": 412,
    "bodySize": 0
  },
  "response": {
    "status": 200,
    "statusText": "OK",
    "httpVersion": "HTTP/2",
    "mimeType": "text/html",
    "headers": {
      "content-type": ["text/html; charset=utf-8"]
    },
    "headersSize": 820,
    "bodySize": 41325,
    "fromCache": false
  },
  "timing": {
    "startedAt": "2026-06-08T10:15:30.123Z",
    "durationMs": 143.7,
    "blockedMs": 0.4,
    "dnsMs": 0,
    "connectMs": 0,
    "sslMs": 0,
    "sendMs": 1.2,
    "waitMs": 95.4,
    "receiveMs": 46.7
  },
  "browser": {
    "resourceType": "fetch",
    "priority": "High",
    "connectionId": "125636",
    "initiator": {
      "type": "script",
      "topFunction": "a.cg.a.cg.fetch",
      "topUrl": "https://github.githubassets.com/assets/fetch-utilities-f1b06a7448671df6.js",
      "topLine": 2,
      "topColumn": 5570,
      "stackDepth": 41
    }
  }
}
```

Normalization details:

- `durationMs` is `entry.time` when it is a non-negative number.
- `startedAt` is `entry.startedDateTime`.
- HAR timing fields with value `-1` mean unavailable and must be omitted from
  the normalized record.
- Header names are lowercased. Header values are arrays because OTel header
  attributes use string arrays.
- `urlSanitized` must remove URL credentials and redact known sensitive query
  parameters.
- Bodies are not stored or exported by default. If retained later, store them
  outside spans and add a content hash/reference only.
- `_initiator.stack` is high-cardinality and can expose source URLs. Keep only
  the top frame and aggregate counts in spans. Store the full raw HAR separately
  only behind an explicit retention policy.

## OTel trace model

Emit one HTTP `CLIENT` span per normalized network request. Optionally emit a
page/navigation root span when the ingestor has enough entries to compute page
start and end.

Resource attributes:

| Attribute | Value |
| --- | --- |
| `service.name` | `hosho-browser` |
| `service.namespace` | `hosho` |
| `service.instance.id` | `capture.sessionId` |
| `telemetry.sdk.name` | `hosho` |
| `telemetry.sdk.language` | `webjs` |

Scope:

| Field | Value |
| --- | --- |
| `scope.name` | `hosho.har` |
| `scope.version` | extension or ingestor version |

## Trace and span identity

Prefer deterministic IDs so reprocessing the same HAR does not duplicate data.

Trace ID selection:

1. If a valid outbound W3C `traceparent` header is present and Hosho is the only
   browser-span source, use its trace ID.
2. Otherwise, use a deterministic 16-byte ID derived from
   `sha256("hosho.trace.v1" + sessionId + pageId)`.
3. If `sessionId` or `pageId` is missing, derive the trace ID from
   `sha256("hosho.trace.v1" + url.origin + startedAt rounded to navigation)`.

Span ID selection:

1. If a valid outbound `traceparent` header is present and Hosho is filling in
   the missing browser client span, use the header parent ID as this span ID.
   Backend server spans that consumed that header will then attach beneath this
   synthetic client span.
2. Otherwise, use a deterministic 8-byte ID derived from
   `sha256("hosho.span.v1" + dedupeKey)`.

Parent selection:

- In page-trace mode, request spans without `traceparent` should have the
  page/navigation span as parent.
- In traceparent-stitch mode, request spans usually have no page parent because
  they join the backend trace. Keep `hosho.capture.page_id` as an attribute for
  page-level grouping.
- If real browser OTel instrumentation already emits HTTP client spans, do not
  also synthesize HAR spans with the same `traceparent` span ID. Either drop the
  HAR span or emit it as an observe-only span with a link to the `traceparent`
  context.

Always store `identity.dedupeKey` and export it as `hosho.har.dedupe_key`.

## Request span mapping

OTLP span:

| Span field | Value |
| --- | --- |
| `name` | `{METHOD}` unless a low-cardinality `url.template` is known |
| `kind` | `3` (`SPAN_KIND_CLIENT`) |
| `traceId` | selected by trace ID rules |
| `spanId` | selected by span ID rules |
| `parentSpanId` | page root span ID when applicable |
| `startTimeUnixNano` | `startedAt` converted to Unix ns |
| `endTimeUnixNano` | `startedAt + durationMs` converted to Unix ns |

Standard attributes:

| OTel attribute | Source |
| --- | --- |
| `http.request.method` | normalized request method, or `_OTHER` |
| `http.request.method_original` | original method when method is mapped to `_OTHER` or canonicalized |
| `url.full` | `request.urlSanitized` |
| `url.scheme` | parsed URL scheme |
| `server.address` | parsed host |
| `server.port` | parsed port, defaulting to 80/443 by scheme |
| `http.response.status_code` | `response.status` when present and greater than 0 |
| `network.protocol.name` | `http` when protocol version is known |
| `network.protocol.version` | `1.0`, `1.1`, `2`, or `3` parsed from HAR HTTP version |
| `http.request.body.size` | `request.bodySize` when non-negative |
| `http.request.size` | `request.headersSize + request.bodySize` when both are non-negative |
| `http.response.body.size` | `response.bodySize` when non-negative |
| `http.response.size` | `response.headersSize + response.bodySize` when both are non-negative |
| `user_agent.original` | request `user-agent` header if explicitly enabled |

Hosho attributes:

| Attribute | Source |
| --- | --- |
| `hosho.schema` | `hosho.network.request.v1` |
| `hosho.capture.source` | `capture.source` |
| `hosho.capture.session_id` | `capture.sessionId` |
| `hosho.capture.page_id` | `capture.pageId` |
| `hosho.browser.tab_id` | `capture.tabId` |
| `hosho.har.entry_hash` | `identity.entryHash` |
| `hosho.har.dedupe_key` | `identity.dedupeKey` |
| `hosho.har.connection_id` | `browser.connectionId` |
| `hosho.har.resource_type` | `browser.resourceType` |
| `hosho.har.priority` | `browser.priority` |
| `hosho.har.timing.blocked_ms` | `timing.blockedMs` |
| `hosho.har.timing.dns_ms` | `timing.dnsMs` |
| `hosho.har.timing.connect_ms` | `timing.connectMs` |
| `hosho.har.timing.ssl_ms` | `timing.sslMs` |
| `hosho.har.timing.send_ms` | `timing.sendMs` |
| `hosho.har.timing.wait_ms` | `timing.waitMs` |
| `hosho.har.timing.receive_ms` | `timing.receiveMs` |
| `code.function` | `browser.initiator.topFunction` |
| `code.file.path` | `browser.initiator.topUrl` |
| `code.line.number` | `browser.initiator.topLine` |
| `code.column.number` | `browser.initiator.topColumn` |

Header export policy:

- Do not export all headers.
- Allowlist useful low-risk headers such as `content-type`, `accept`, and
  `traceparent`.
- Export request headers as `http.request.header.<name>` string arrays.
- Export response headers as `http.response.header.<name>` string arrays.
- Never export `authorization`, `cookie`, `set-cookie`, or request/response body
  text by default.

## Span status and errors

Set OTel span status from the HTTP client perspective:

- Status code 100-399: leave OTel status unset.
- Status code 400-599: set OTel status code to `2` (`STATUS_CODE_ERROR`) and set
  `error.type` to the status code string, for example `"404"`.
- Missing status, status `0`, negative duration, or malformed URL: set OTel
  status code to `2` and set `error.type` to a low-cardinality value such as
  `network_error`, `invalid_har`, or `malformed_url`.
- Do not set a status description when `http.response.status_code` is enough.

## Optional page/navigation span

When the ingestor sees a batch or has per-page buffering, emit one internal root
span:

| Span field | Value |
| --- | --- |
| `name` | `browser.navigation` |
| `kind` | `1` (`SPAN_KIND_INTERNAL`) |
| `startTimeUnixNano` | minimum request start |
| `endTimeUnixNano` | maximum request end |

Attributes:

| Attribute | Value |
| --- | --- |
| `hosho.capture.page_id` | page ID |
| `url.full` | sanitized page URL |
| `hosho.navigation.request_count` | number of request spans |
| `hosho.navigation.error_count` | request spans with error status |

Do not force traceparent-stitched request spans under this page span if they use
backend trace IDs. Use `hosho.capture.page_id` for joining page and backend
views at query time.

## OTLP JSON shape

OTLP JSON uses lowerCamelCase protobuf field names. `traceId` and `spanId` are
hex strings. Enum fields use integer values.

Minimal request span example:

```json
{
  "resourceSpans": [
    {
      "resource": {
        "attributes": [
          {"key": "service.name", "value": {"stringValue": "hosho-browser"}},
          {"key": "service.namespace", "value": {"stringValue": "hosho"}},
          {"key": "service.instance.id", "value": {"stringValue": "018f6fd6-4a12-7a4a-b2bc-1ff0fef32b0c"}}
        ]
      },
      "scopeSpans": [
        {
          "scope": {"name": "hosho.har", "version": "0.1.0"},
          "spans": [
            {
              "traceId": "4bf92f3577b34da6a3ce929d0e0e4736",
              "spanId": "00f067aa0ba902b7",
              "name": "GET",
              "kind": 3,
              "startTimeUnixNano": "1780913730123000000",
              "endTimeUnixNano": "1780913730266700000",
              "attributes": [
                {"key": "http.request.method", "value": {"stringValue": "GET"}},
                {"key": "url.full", "value": {"stringValue": "https://github.com/org/repo/pull/1/files?short_path=REDACTED"}},
                {"key": "url.scheme", "value": {"stringValue": "https"}},
                {"key": "server.address", "value": {"stringValue": "github.com"}},
                {"key": "server.port", "value": {"intValue": "443"}},
                {"key": "http.response.status_code", "value": {"intValue": "200"}},
                {"key": "network.protocol.name", "value": {"stringValue": "http"}},
                {"key": "network.protocol.version", "value": {"stringValue": "2"}},
                {"key": "hosho.capture.page_id", "value": {"stringValue": "tab-125:nav-4"}},
                {"key": "hosho.har.timing.wait_ms", "value": {"doubleValue": 95.4}}
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

## Implementation order

1. Add envelope detection and raw HAR entry normalization.
2. Add deterministic `entryHash` and `dedupeKey`.
3. Add URL, header, timing, initiator, and traceparent extraction.
4. Generate one OTLP request span per HAR entry.
5. Export to an OTLP HTTP endpoint or log OTLP JSON while wiring storage.
6. Add page/navigation root spans once the extension sends page/session IDs and
   a navigation flush signal.
