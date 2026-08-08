// decodePng unit test (issue #259): verifies the zero-dep PNG decoder used by
// the pixel comparison tool. Builds a small RGBA PNG whose rows exercise
// filters 0-4 and asserts the decoded pixels match the original.
import assert from "node:assert/strict";
import { deflateSync } from "node:zlib";
import test from "node:test";
import { decodePng } from "../ui_e2e/png_util.mjs";

function crc32(buf) {
  let table = crc32.table;
  if (!table) {
    table = crc32.table = new Int32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
      table[n] = c;
    }
  }
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = table[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

function makePng(pixels, w, h) {
  const channels = 4;
  const stride = w * channels;
  const raw = Buffer.alloc(h * (stride + 1));
  let prev = Buffer.alloc(stride);
  for (let y = 0; y < h; y++) {
    const filter = y % 5; // exercise filters 0..4
    raw[y * (stride + 1)] = filter;
    for (let x = 0; x < stride; x++) {
      const orig = pixels[y * stride + x];
      const a = x >= channels ? pixels[y * stride + x - channels] : 0;
      const b = x >= channels ? prev[x - channels] : prev[x]; // b = up (same index)
      const up = prev[x];
      const c = x >= channels ? prev[x - channels] : 0;
      let v = orig;
      if (filter === 1) v = (orig - a) & 0xff;
      else if (filter === 2) v = (orig - up) & 0xff;
      else if (filter === 3) v = (orig - ((a + up) >> 1)) & 0xff;
      else if (filter === 4) {
        const p = a + up - c, pa = Math.abs(p - a), pb = Math.abs(p - up), pc = Math.abs(p - c);
        const pred = pa <= pb && pa <= pc ? a : pb <= pc ? up : c;
        v = (orig - pred) & 0xff;
      }
      raw[y * (stride + 1) + 1 + x] = v;
    }
    for (let x = 0; x < stride; x++) prev[x] = pixels[y * stride + x];
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0); ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw)),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

test("decodePng round-trips RGBA with filters 0-4", () => {
  const w = 4, h = 5;
  const pixels = Buffer.alloc(w * h * 4);
  for (let i = 0; i < pixels.length; i++) pixels[i] = (i * 37 + 11) & 0xff;
  const png = makePng(pixels, w, h);
  const out = decodePng(png);
  assert.equal(out.width, w);
  assert.equal(out.height, h);
  assert.deepEqual(out.data, pixels);
});

test("decodePng rejects interlaced PNG", () => {
  const w = 2, h = 2;
  const pixels = Buffer.alloc(w * h * 4, 0);
  const png = makePng(pixels, w, h);
  // flip IHDR interlace byte (offset: signature 8 + chunk header 8 + 12 bytes in)
  png[8 + 8 + 12] = 1;
  assert.throws(() => decodePng(png), /interlaced/);
});
