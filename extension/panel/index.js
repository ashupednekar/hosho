import { startPanel } from "./app.js";

startPanel({
  chromeDevtools: chrome.devtools,
  output: document.getElementById("output"),
  errorList: document.getElementById("errors"),
  status: document.getElementById("status"),
});
