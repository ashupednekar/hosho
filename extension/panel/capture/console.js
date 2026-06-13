import {
  CONSOLE_BUFFER_NAME,
  CONSOLE_DRAIN_INTERVAL_MS,
  CONSOLE_INSTALLED_FLAG,
  ENDPOINTS,
} from "../config.js";
import { sanitizedUrl } from "./url.js";

export function installConsoleCapture(chromeDevtools) {
  chromeDevtools.inspectedWindow.eval(`(() => {
    if (window.${CONSOLE_INSTALLED_FLAG}) return;
    window.${CONSOLE_INSTALLED_FLAG} = true;
    window.${CONSOLE_BUFFER_NAME} = [];

    const push = (level, args) => {
      window.${CONSOLE_BUFFER_NAME}.push({
        level,
        args: args.map((arg) => {
          try {
            if (arg instanceof Error) {
              return { name: arg.name, message: arg.message, stack: arg.stack };
            }
            if (typeof arg === "string") return arg;
            return JSON.stringify(arg);
          } catch (_) {
            return String(arg);
          }
        }),
        url: location.href,
        capturedAt: new Date().toISOString()
      });
    };

    for (const level of ["debug", "log", "info", "warn", "error"]) {
      const original = console[level];
      console[level] = function(...args) {
        push(level, args);
        return original.apply(this, args);
      };
    }

    window.addEventListener("error", (event) => {
      push("error", [event.error || event.message]);
    });

    window.addEventListener("unhandledrejection", (event) => {
      push("error", [event.reason || "Unhandled promise rejection"]);
    });
  })()`);
}

export function startConsoleDrain({ chromeDevtools, sendJson, render }) {
  setInterval(() => {
    chromeDevtools.inspectedWindow.eval(`(() => {
      const events = window.${CONSOLE_BUFFER_NAME} || [];
      window.${CONSOLE_BUFFER_NAME} = [];
      return events;
    })()`, (events, error) => {
      if (error || !Array.isArray(events) || events.length === 0) return;

      for (const event of events) {
        const sanitizedEvent = sanitizeConsoleEvent(event);
        sendJson(ENDPOINTS.log, sanitizedEvent);

        render(sanitizedEvent.level, sanitizedEvent);
      }
    });
  }, CONSOLE_DRAIN_INTERVAL_MS);
}

function sanitizeConsoleEvent(event) {
  return {
    ...event,
    url: sanitizedUrl(event.url),
  };
}
