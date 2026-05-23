const SENSITIVE_NAMES = new Set([
  "authorization",
  "cookie",
  "set-cookie",
  "x-api-key",
  "token",
  "access_token",
  "refresh_token",
  "secret",
  "key",
]);

export function redactNetworkEntry(entry) {
  const copy = JSON.parse(JSON.stringify(entry));

  redactHeaders(copy.request?.headers);
  redactHeaders(copy.response?.headers);
  redactQueryString(copy.request?.queryString);

  if (copy.request?.postData?.text) {
    copy.request.postData.text = "[redacted]";
  }

  return copy;
}

function redactHeaders(headers = []) {
  for (const header of headers) {
    if (isSensitiveName(header.name)) {
      header.value = "[redacted]";
    }
  }
}

function redactQueryString(params = []) {
  for (const param of params) {
    if (isSensitiveName(param.name)) {
      param.value = "[redacted]";
    }
  }
}

function isSensitiveName(name = "") {
  return SENSITIVE_NAMES.has(name.toLowerCase());
}
