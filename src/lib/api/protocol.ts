// URLs served by the Rust `mantella://` scheme handler (see src-tauri/src/pdf/protocol.rs).
// On Windows, WebView2 exposes custom schemes as http://<scheme>.localhost.
const isWindows = navigator.userAgent.includes("Windows");
const BASE = isWindows ? "http://mantella.localhost" : "mantella://localhost";

export function pageUrl(docId: number, pageIndex: number, pixelWidth: number): string {
  return `${BASE}/${docId}/${pageIndex}?w=${Math.max(16, Math.round(pixelWidth))}`;
}
