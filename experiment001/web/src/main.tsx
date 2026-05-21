// Polyfill Node's Buffer global before any solana/anchor code evaluates —
// `@solana/spl-token` and `@coral-xyz/anchor` both reference Buffer at the
// module top level. Without this, the dashboard chunk crashes the moment
// React lazy-resolves it.
import { Buffer } from "buffer";
if (typeof (globalThis as { Buffer?: unknown }).Buffer === "undefined") {
  (globalThis as { Buffer: typeof Buffer }).Buffer = Buffer;
}

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
