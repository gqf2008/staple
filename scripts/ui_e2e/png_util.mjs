// Minimal PNG decode (issue #259) used by pixel_compare.mjs and its unit test.
// Supports 8-bit, non-interlaced, color types 0/2/6 (gray/RGB/RGBA).
import { inflateSync } from "node:zlib";

export function decodePng(buf) {
  if (buf.readUInt32BE(0) !== 0x89504e47) throw new Error("not a png");
  let w = 0, h = 0, bit = 0, ctype = 0, interlace = 0;
  const idat = [];
  let off = 8;
  while (off < buf.length) {
    const len = buf.readUInt32BE(off);
    const type = buf.toString("ascii", off + 4, off + 8);
    const data = buf.subarray(off + 8, off + 8 + len);
    if (type === "IHDR") {
      w = data.readUInt32BE(0); h = data.readUInt32BE(4);
      bit = data[8]; ctype = data[9]; interlace = data[12];
    } else if (type === "IDAT") {
      idat.push(data);
    } else if (type === "IEND") {
      break;
    }
    off += 12 + len;
  }
  if (bit !== 8 || ![0, 2, 6].includes(ctype)) throw new Error(`unsupported png bit=${bit} ctype=${ctype}`);
  if (interlace !== 0) throw new Error("interlaced png not supported (Playwright output is non-interlaced)");
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
