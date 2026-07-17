#!/usr/bin/env node
/**
 * Composes the source app artwork onto the macOS icon grid: a rounded
 * square filling ~824/1024 of the canvas with transparent margins (without
 * which the icon looks oversized in the Dock). No image dependencies.
 * Feed the output to `tauri icon` to produce all platform icon sizes.
 *
 * Usage: node scripts/gen-icon.mjs <source.png> <output.png>
 *
 * The source must be a square, non-interlaced, 8-bit RGB/RGBA PNG; it is
 * scaled to cover the rounded square exactly.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { deflateSync, inflateSync } from "node:zlib";

const SIZE = 1024;
const MARGIN = 100;
const CONTENT = SIZE - 2 * MARGIN;
const CORNER_R = 185;

const [src, out] = process.argv.slice(2);
if (!src || !out) {
  console.error("usage: gen-icon.mjs <source.png> <output.png>");
  process.exit(1);
}

// --- minimal PNG decoder (8-bit RGB/RGBA, no interlace) ---

function decodePng(buf) {
  if (buf.readUInt32BE(0) !== 0x89504e47) throw new Error("not a PNG");
  let pos = 8;
  let width = 0;
  let height = 0;
  let channels = 0;
  const idat = [];
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos);
    const type = buf.toString("ascii", pos + 4, pos + 8);
    const data = buf.subarray(pos + 8, pos + 8 + len);
    if (type === "IHDR") {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      const depth = data[8];
      const color = data[9];
      const interlace = data[12];
      if (depth !== 8 || (color !== 2 && color !== 6) || interlace !== 0) {
        throw new Error("unsupported PNG: need 8-bit RGB/RGBA, no interlace");
      }
      channels = color === 6 ? 4 : 3;
    } else if (type === "IDAT") {
      idat.push(data);
    } else if (type === "IEND") {
      break;
    }
    pos += 12 + len;
  }

  const raw = inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const px = Buffer.alloc(width * height * channels);
  const paeth = (a, b, c) => {
    const p = a + b - c;
    const pa = Math.abs(p - a);
    const pb = Math.abs(p - b);
    const pc = Math.abs(p - c);
    return pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
  };
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const row = y * stride;
    const prev = row - stride;
    for (let i = 0; i < stride; i++) {
      const x = raw[y * (stride + 1) + 1 + i];
      const a = i >= channels ? px[row + i - channels] : 0;
      const b = y > 0 ? px[prev + i] : 0;
      const c = y > 0 && i >= channels ? px[prev + i - channels] : 0;
      let v;
      if (filter === 0) v = x;
      else if (filter === 1) v = x + a;
      else if (filter === 2) v = x + b;
      else if (filter === 3) v = x + ((a + b) >> 1);
      else if (filter === 4) v = x + paeth(a, b, c);
      else throw new Error(`bad PNG filter ${filter}`);
      px[row + i] = v & 0xff;
    }
  }
  return { width, height, channels, px };
}

const source = decodePng(readFileSync(src));
if (source.width !== source.height) throw new Error("source must be square");

function roundedRectAlpha(x, y, x0, y0, x1, y1, r) {
  const cx = Math.max(x0 + r, Math.min(x, x1 - r));
  const cy = Math.max(y0 + r, Math.min(y, y1 - r));
  if (x >= x0 + r && x <= x1 - r)
    return x >= x0 && x <= x1 && y >= y0 && y <= y1 ? 1 : 0;
  if (y >= y0 + r && y <= y1 - r)
    return x >= x0 && x <= x1 && y >= y0 && y <= y1 ? 1 : 0;
  const d = Math.hypot(x - cx, y - cy);
  return Math.max(0, Math.min(1, r - d + 0.5));
}

// box-sample the source at a canvas position (source covers the content square)
const SS = 3;
function sampleSource(x, y) {
  const scale = source.width / CONTENT;
  let r = 0;
  let g = 0;
  let b = 0;
  for (let sy = 0; sy < SS; sy++) {
    for (let sx = 0; sx < SS; sx++) {
      const u = (x + (sx + 0.5) / SS - MARGIN) * scale;
      const v = (y + (sy + 0.5) / SS - MARGIN) * scale;
      const ix = Math.max(0, Math.min(source.width - 1, Math.floor(u)));
      const iy = Math.max(0, Math.min(source.height - 1, Math.floor(v)));
      const o = (iy * source.width + ix) * source.channels;
      r += source.px[o];
      g += source.px[o + 1];
      b += source.px[o + 2];
    }
  }
  return [r / (SS * SS), g / (SS * SS), b / (SS * SS)];
}

const px = Buffer.alloc(SIZE * SIZE * 4);
for (let y = 0; y < SIZE; y++) {
  for (let x = 0; x < SIZE; x++) {
    const i = (y * SIZE + x) * 4;
    const a = roundedRectAlpha(
      x,
      y,
      MARGIN,
      MARGIN,
      SIZE - 1 - MARGIN,
      SIZE - 1 - MARGIN,
      CORNER_R,
    );
    if (a > 0) {
      const [r, g, b] = sampleSource(x, y);
      px[i] = Math.round(r);
      px[i + 1] = Math.round(g);
      px[i + 2] = Math.round(b);
    }
    px[i + 3] = Math.round(a * 255);
  }
}

// --- minimal PNG encoder ---
function crc32(buf) {
  let c,
    crc = 0xffffffff;
  for (let n = 0; n < buf.length; n++) {
    c = (crc ^ buf[n]) & 0xff;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    crc = (crc >>> 8) ^ c;
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // RGBA

const raw = Buffer.alloc((SIZE * 4 + 1) * SIZE);
for (let y = 0; y < SIZE; y++) {
  raw[y * (SIZE * 4 + 1)] = 0; // filter: none
  px.copy(raw, y * (SIZE * 4 + 1) + 1, y * SIZE * 4, (y + 1) * SIZE * 4);
}

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

writeFileSync(out, png);
console.log(`icon written to ${out}`);
