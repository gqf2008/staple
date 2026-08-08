// Pixel-level UI comparison (issue #259): upstream Storybook elements vs
// Staple elements. Requires both servers running:
//   - upstream Storybook on 6006 (reference mirror ui/, `pnpm storybook --no-open`)
//   - Staple on 3100 (`make dev` / demo)
// Env: BASE_URL (default http://127.0.0.1:3100), UPSTREAM_URL (default
// http://127.0.0.1:6006), PW_EXECUTABLE (chromium), E2E_OUT_DIR (default
// target/ui-pixel-compare), E2E_REPORT (default .../report.json).
// Zero runtime deps: PNG decode uses node:zlib; playwright must be available
// (NODE_PATH=$(npm root -g)).
import { createRequire } from "node:module";
import { mkdirSync, writeFileSync } from "node:fs";
import { inflateSync } from "node:zlib";
import { join } from "node:path";

const require = createRequire(import.meta.url);
const { chromium } = require("playwright");

const UP = process.env.UPSTREAM_URL || "http://127.0.0.1:6006";
const ST = process.env.BASE_URL || "http://127.0.0.1:3100";
const EXE = process.env.PW_EXECUTABLE || undefined;
const OUT = process.env.E2E_OUT_DIR || "target/ui-pixel-compare";
const REPORT = process.env.E2E_REPORT || join(OUT, "report.json");
mkdirSync(OUT, { recursive: true });

// --- minimal PNG decode (8-bit, non-interlaced; color types 0/2/6) ---
function decodePng(buf) {
  if (buf.readUInt32BE(0) !== 0x89504e47) throw new Error("not a png");
  let w = 0, h = 0, bit = 0, ctype = 0;
  const idat = [];
  let off = 8;
  while (off < buf.length) {
    const len = buf.readUInt32BE(off);
    const type = buf.toString("ascii", off + 4, off + 8);
    const data = buf.subarray(off + 8, off + 8 + len);
    if (type === "IHDR") { w = data.readUInt32BE(0); h = data.readUInt32BE(4); bit = data[8]; ctype = data[9]; }
    else if (type === "IDAT") idat.push(data);
    else if (type === "IEND") break;
    off += 12 + len;
  }
  if (bit !== 8 || ![0, 2, 6].includes(ctype)) throw new Error(`unsupported png bit=${bit} ctype=${ctype}`);
  const channels = ctype === 6 ? 4 : ctype === 2 ? 3 : 1;
  const raw = inflateSync(Buffer.concat(idat));
  const stride = w * channels;
  const out = Buffer.alloc(w * h * 4);
  let prev = Buffer.alloc(stride);
  for (let y = 0; y < h; y++) {
    const filter = raw[y * (stride + 1)];
    const line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    const cur = Buffer.alloc(stride);
    for (let x = 0; x < stride; x++) {
      const a = x >= channels ? cur[x - channels] : 0;
      const b = prev[x];
      const c = x >= channels ? prev[x - channels] : 0;
      let v = line[x];
      if (filter === 1) v = (v + a) & 0xff;
      else if (filter === 2) v = (v + b) & 0xff;
      else if (filter === 3) v = (v + ((a + b) >> 1)) & 0xff;
      else if (filter === 4) {
        const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
        v = (v + (pa <= pb && pa <= pc ? a : pb <= pc ? b : c)) & 0xff;
      }
      cur[x] = v;
    }
    for (let x = 0; x < stride; x += channels) {
      const di = (y * w + x / channels) * 4;
      out[di] = cur[x];
      out[di + 1] = channels > 1 ? cur[x + 1] : cur[x];
      out[di + 2] = channels > 2 ? cur[x + 2] : cur[x];
      out[di + 3] = channels > 3 ? cur[x + 3] : 255;
    }
    prev = cur;
  }
  return { width: w, height: h, data: out };
}

function resizeNearestH(png, targetH) {
  const scale = targetH / png.height;
  const w = Math.max(1, Math.round(png.width * scale));
  const out = { width: w, height: targetH, data: Buffer.alloc(w * targetH * 4) };
  for (let y = 0; y < targetH; y++) {
    const sy = Math.min(png.height - 1, Math.floor(y / scale));
    for (let x = 0; x < w; x++) {
      const sx = Math.min(png.width - 1, Math.floor(x / scale));
      const si = (sy * png.width + sx) * 4, di = (y * w + x) * 4;
      for (let c = 0; c < 4; c++) out.data[di + c] = png.data[si + c];
    }
  }
  return out;
}

function intersection(a, b) {
  const w = Math.min(a.width, b.width), h = Math.min(a.height, b.height);
  const pa = { width: w, height: h, data: Buffer.alloc(w * h * 4) };
  const pb = { width: w, height: h, data: Buffer.alloc(w * h * 4) };
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const ai = (y * a.width + x) * 4, bi = (y * b.width + x) * 4, di = (y * w + x) * 4;
    for (let c = 0; c < 4; c++) { pa.data[di + c] = a.data[ai + c]; pb.data[di + c] = b.data[bi + c]; }
  }
  return [pa, pb];
}

function metrics(a, b) {
  const w = a.width, h = a.height;
  let sum = 0, diffPix = 0, total = 0, rsum = 0, rtotal = 0;
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const i = (y * w + x) * 4;
    const dr = Math.abs(a.data[i] - b.data[i]), dg = Math.abs(a.data[i + 1] - b.data[i + 1]), db = Math.abs(a.data[i + 2] - b.data[i + 2]);
    const d = (dr + dg + db) / 3;
    sum += d; total++;
    if (dr > 24 || dg > 24 || db > 24) diffPix++;
    const ring = Math.min(Math.min(x, w - 1 - x) / w, Math.min(y, h - 1 - y) / h) < 0.25;
    if (ring) { rsum += d; rtotal++; }
  }
  return {
    w, h,
    meanAbsDiff: +(sum / total).toFixed(2),
    ringMeanAbsDiff: rtotal ? +(rsum / rtotal).toFixed(2) : 0,
    pctDiffPix: +((diffPix / total) * 100).toFixed(2),
  };
}

function bgColor(png) {
  const w = png.width, h = png.height;
  let r = 0, g = 0, b = 0, n = 0;
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const ring = Math.min(Math.min(x, w - 1 - x) / w, Math.min(y, h - 1 - y) / h) < 0.25;
    if (!ring) continue;
    const i = (y * w + x) * 4;
    r += png.data[i]; g += png.data[i + 1]; b += png.data[i + 2]; n++;
  }
  return n ? `rgb(${Math.round(r / n)},${Math.round(g / n)},${Math.round(b / n)})` : "n/a";
}

const verdict = (m) => (m.ringMeanAbsDiff < 10 ? "ALIGN" : m.ringMeanAbsDiff < 40 ? "CLOSE" : "DIFF");

const CID = process.env.E2E_COMPANY_ID || "48e56bc1-17bf-4195-bd2a-b59bd490e7aa"; // demo company (fallback)

const pairs = [
  { name: "button-default", up: { id: "foundations-primitive-matrix--all-primitives", selector: "button.bg-primary, button[class*='bg-primary']" }, st: { url: `${ST}/companies/${CID}/board/chat`, selector: 'button[type="submit"]' } },
  { name: "badge-secondary", up: { id: "foundations-primitive-matrix--all-primitives", selector: "span.bg-secondary, span[class*='bg-secondary']" }, st: { url: `${ST}/companies/${CID}/issues`, selector: ".badge" } },
  { name: "card", up: { id: "foundations-primitive-matrix--all-primitives", selector: "div.rounded-lg.border.bg-card, div[class*='rounded-lg'][class*='border'][class*='bg-card']" }, st: { url: `${ST}/issues/613a4977-20d0-4820-b146-8e8d5924ebfa`, selector: ".issue-section" } },
  { name: "command-palette", up: { id: "foundations-primitive-coverage--command-palette-inline", selector: "[cmdk-root], [class*='max-w-md']" }, st: { url: `${ST}/companies/${CID}/board`, selector: ".command-palette-panel", openPalette: true } },
  { name: "sidebar", up: { id: "product-navigation-layout--board-chrome-matrix", selector: "div[class*='w-60']" }, st: { url: `${ST}/companies/${CID}/board`, selector: ".app-sidebar" } },
  { name: "board-card", up: { id: "paperclip-successful-run-handoff--issue-card-indicator", selector: "[class*='cursor-grab']" }, st: { url: `${ST}/companies/${CID}/board`, selector: ".board-card" } },
];

const browser = await (EXE ? chromium.launch({ executablePath: EXE }) : chromium.launch());
const report = [];
for (const pair of pairs) {
  const row = { name: pair.name };
  try {
    const up = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    await up.goto(`${UP}/iframe.html?id=${pair.up.id}&viewMode=story&globals=theme:light`, { waitUntil: "domcontentloaded" });
    await up.waitForTimeout(1500);
    const upLoc = up.locator(pair.up.selector).first();
    if (!(await upLoc.count())) { row.error = `upstream selector not found: ${pair.up.selector}`; await up.close(); report.push(row); continue; }
    const bb = await upLoc.boundingBox();
    row.upSize = `${Math.round(bb.width)}x${Math.round(bb.height)}`;
    const upBuf = await upLoc.screenshot();
    await up.close();

    const st = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    await st.goto(pair.st.url, { waitUntil: "networkidle" });
    await st.waitForTimeout(500);
    if (pair.st.openPalette) { await st.keyboard.press("Meta+k"); await st.waitForTimeout(400); }
    const stLoc = st.locator(pair.st.selector).first();
    if (!(await stLoc.count())) { row.error = `staple selector not found: ${pair.st.selector}`; await st.close(); report.push(row); continue; }
    const sbb = await stLoc.boundingBox();
    row.stSize = `${Math.round(sbb.width)}x${Math.round(sbb.height)}`;
    const stBuf = await stLoc.screenshot();
    await st.close();

    const [a, b] = intersection(resizeNearestH(decodePng(upBuf), 100), resizeNearestH(decodePng(stBuf), 100));
    row.metrics = metrics(a, b);
    row.ring = { upBg: bgColor(a), stBg: bgColor(b) };
    row.verdict = verdict(row.metrics);
    writeFileSync(join(OUT, `${pair.name}-up.png`), upBuf);
    writeFileSync(join(OUT, `${pair.name}-st.png`), stBuf);
  } catch (e) { row.error = String(e).slice(0, 300); }
  report.push(row);
}
await browser.close();
writeFileSync(REPORT, JSON.stringify({ generatedAt: new Date().toISOString(), upstream: UP, staple: ST, results: report }, null, 2));
console.log(JSON.stringify(report, null, 2));
