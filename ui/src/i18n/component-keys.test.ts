import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import en from "./locales/en.json";

function hasPath(d: unknown, parts: string[]): boolean {
  let cur: unknown = d;
  for (const p of parts) {
    if (typeof cur !== "object" || cur === null || !(p in (cur as Record<string, unknown>))) {
      return false;
    }
    cur = (cur as Record<string, unknown>)[p];
  }
  return true;
}

function walkTsx(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    if (name === "node_modules") continue;
    const full = join(dir, name);
    const st = statSync(full);
    if (st.isDirectory()) {
      out.push(...walkTsx(full));
    } else if (name.endsWith(".tsx") && !name.includes(".test.")) {
      out.push(full);
    }
  }
  return out;
}

function resolveKey(key: string): boolean {
  const parts = key.split(".");
  if (hasPath(en, parts)) return true;
  const last = parts[parts.length - 1];
  if (last.endsWith("_one") || last.endsWith("_other")) {
    return hasPath(en, [...parts.slice(0, -1), last.slice(0, -4)]);
  }
  // plural base keys resolve through their _one/_other siblings
  const parent = parts.slice(0, -1);
  if (hasPath(en, [...parent, last + "_one"]) || hasPath(en, [...parent, last + "_other"])) {
    return true;
  }
  // dynamic keys: allow when the longest static prefix exists (e.g. `status.${status}`)
  const staticParts: string[] = [];
  for (const part of parts) {
    if (part.startsWith("${") || part.startsWith("`")) break;
    staticParts.push(part);
  }
  return staticParts.length >= 2 && hasPath(en, staticParts);
}

describe("component i18n keys resolve against en.json", () => {
  it("every static t() key used by components exists in the English resource", () => {
    const files = walkTsx(join(__dirname, ".."));
    const keyRe = /\bt\(\s*"([^"]+)"/g;
    const missing = new Map<string, string[]>();

    for (const file of files) {
      const src = readFileSync(file, "utf8");
      for (const match of src.matchAll(keyRe)) {
        const key = match[1]!;
        if (!resolveKey(key)) {
          const rel = file.slice(file.indexOf("src") + 4);
          missing.set(rel, [...(missing.get(rel) ?? []), key]);
        }
      }
    }

    const total = [...missing.values()].reduce((sum, keys) => sum + keys.length, 0);
    expect(total, `unresolvable t() keys:\n${[...missing.entries()]
      .map(([file, keys]) => `${file}:\n    ${keys.join("\n    ")}`)
      .join("\n")}`).toBe(0);
  });
});
