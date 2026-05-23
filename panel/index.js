import { startPanel } from "./app.js";

startPanel({
  chromeDevtools: chrome.devtools,
  output: document.getElementById("output"),
});
