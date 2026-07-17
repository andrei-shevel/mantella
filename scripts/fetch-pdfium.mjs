#!/usr/bin/env node
/**
 * Downloads a prebuilt pdfium dynamic library from bblanchon/pdfium-binaries
 * into src-tauri/resources/pdfium/ so the Rust backend can bind to it.
 *
 * Usage:
 *   node scripts/fetch-pdfium.mjs             # auto-detect current platform
 *   node scripts/fetch-pdfium.mjs --target mac-arm64|mac-x64|win-x64|linux-x64|linux-arm64
 */
import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  rmSync,
  copyFileSync,
  existsSync,
  mkdtempSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const TARGETS = {
  "mac-arm64": { archive: "pdfium-mac-arm64.tgz", lib: "lib/libpdfium.dylib" },
  "mac-x64": { archive: "pdfium-mac-x64.tgz", lib: "lib/libpdfium.dylib" },
  "win-x64": { archive: "pdfium-win-x64.tgz", lib: "bin/pdfium.dll" },
  "linux-x64": { archive: "pdfium-linux-x64.tgz", lib: "lib/libpdfium.so" },
  "linux-arm64": { archive: "pdfium-linux-arm64.tgz", lib: "lib/libpdfium.so" },
};

function detectTarget() {
  const arch = process.arch === "arm64" ? "arm64" : "x64";
  switch (process.platform) {
    case "darwin":
      return `mac-${arch}`;
    case "win32":
      return "win-x64";
    case "linux":
      return `linux-${arch}`;
    default:
      throw new Error(`Unsupported platform: ${process.platform}`);
  }
}

const argIdx = process.argv.indexOf("--target");
const target = argIdx !== -1 ? process.argv[argIdx + 1] : detectTarget();
const spec = TARGETS[target];
if (!spec) {
  console.error(
    `Unknown target "${target}". Valid: ${Object.keys(TARGETS).join(", ")}`,
  );
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const destDir = join(root, "src-tauri", "resources", "pdfium");
const destLib = join(destDir, spec.lib.split("/").pop());

if (existsSync(destLib) && !process.argv.includes("--force")) {
  console.log(
    `pdfium already present at ${destLib} (use --force to re-download)`,
  );
  process.exit(0);
}

const url = `https://github.com/bblanchon/pdfium-binaries/releases/latest/download/${spec.archive}`;
console.log(`Downloading ${url} ...`);

const work = mkdtempSync(join(tmpdir(), "pdfium-"));
try {
  const archivePath = join(work, spec.archive);
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) throw new Error(`Download failed: HTTP ${res.status}`);
  const { writeFileSync } = await import("node:fs");
  writeFileSync(archivePath, Buffer.from(await res.arrayBuffer()));

  execFileSync("tar", ["-xzf", archivePath, "-C", work]);
  const extracted = join(work, spec.lib);
  if (!existsSync(extracted))
    throw new Error(`Archive did not contain ${spec.lib}`);

  mkdirSync(destDir, { recursive: true });
  copyFileSync(extracted, destLib);
  console.log(`pdfium installed at ${destLib}`);
} finally {
  rmSync(work, { recursive: true, force: true });
}
