// Theme switching behavior tests (issue #242).
//
// Runs the real crates/app/src/ui/theme.js in a Node vm with a minimal DOM
// stub (no jsdom / no node_modules), same pattern as the other suites.
//
//   node --test scripts/tests/*.test.mjs   # or: make js-test

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const THEME_SOURCE = readFileSync(
  new URL("../../crates/app/src/ui/theme.js", import.meta.url),
  "utf8",
);

class ClassList {
  constructor(el) { this._el = el; }
  toggle(name, force) {
    const has = this._el._classes.has(name);
    const should = force === undefined ? !has : Boolean(force);
    if (should && !has) this._el._classes.add(name);
    else if (!should && has) this._el._classes.delete(name);
    return should;
  }
  contains(name) { return this._el._classes.has(name); }
}

class Element {
  constructor() {
    this._classes = new Set();
    this._attrs = new Map();
    this._listeners = new Map();
    this._text = "";
    this.dataset = {};
    this.classList = new ClassList(this);
  }
  setAttribute(name, value) { this._attrs.set(String(name), String(value)); }
  getAttribute(name) { return this._attrs.has(name) ? this._attrs.get(name) : null; }
  addEventListener(type, cb) {
    if (!this._listeners.has(type)) this._listeners.set(type, []);
    this._listeners.get(type).push(cb);
  }
  dispatchEvent(event) {
    for (const cb of this._listeners.get(event.type) || []) cb(event);
  }
  get textContent() { return this._text; }
  set textContent(value) { this._text = String(value); }
}

function makeEnv({ stored = null, systemDark = false, storageThrows = false } = {}) {
  const root = new Element();
  const toggle = new Element();
  toggle.setAttribute("id", "theme-toggle");
  toggle.dataset.themeSystem = "主题：系统";
  toggle.dataset.themeLight = "主题：浅色";
  toggle.dataset.themeDark = "主题：深色";
  const mq = new Element();
  mq._listeners = new Map();
  const state = { dark: systemDark };
  const matchMedia = (query) => ({
    get matches() { return state.dark; },
    addEventListener: (t, cb) => mq.addEventListener(t, cb),
    addListener: (cb) => mq.addEventListener("change", cb),
  });
  const document = {
    _listeners: new Map(),
    documentElement: root,
    _byId: new Map([["theme-toggle", toggle]]),
    addEventListener(type, cb) { if (!this._listeners.has(type)) this._listeners.set(type, []); this._listeners.get(type).push(cb); },
    dispatchEvent(event) { for (const cb of this._listeners.get(event.type) || []) cb(event); },
    getElementById(id) { return this._byId.get(id) || null; },
  };
  const storage = {
    getItem() { if (storageThrows) throw new Error("denied"); return stored; },
    setItem() { if (storageThrows) throw new Error("denied"); },
  };
  const context = vm.createContext({ document, window: { matchMedia }, localStorage: storage });
  vm.runInContext(THEME_SOURCE, context);
  document.dispatchEvent({ type: "DOMContentLoaded" });
  return { root, toggle, mq, state };
}

test("system dark applies dark class", () => {
  const { root, toggle } = makeEnv({ stored: "system", systemDark: true });
  assert.equal(root.classList.contains("dark"), true);
  assert.equal(toggle.getAttribute("aria-label"), "主题：系统");
  assert.equal(toggle.textContent, "\u25d0");
});

test("system light keeps light", () => {
  const { root } = makeEnv({ stored: "system", systemDark: false });
  assert.equal(root.classList.contains("dark"), false);
});

test("stored dark and light are honored", () => {
  assert.equal(makeEnv({ stored: "dark", systemDark: false }).root.classList.contains("dark"), true);
  assert.equal(makeEnv({ stored: "light", systemDark: true }).root.classList.contains("dark"), false);
});

test("click cycles system -> light -> dark with localized labels", () => {
  const { root, toggle } = makeEnv({ stored: "system", systemDark: false });
  toggle.dispatchEvent({ type: "click" }); // -> light
  assert.equal(root.classList.contains("dark"), false);
  assert.equal(toggle.getAttribute("aria-label"), "主题：浅色");
  assert.equal(toggle.textContent, "\u2600");
  toggle.dispatchEvent({ type: "click" }); // -> dark
  assert.equal(root.classList.contains("dark"), true);
  assert.equal(toggle.getAttribute("aria-label"), "主题：深色");
  assert.equal(toggle.textContent, "\u263e");
  toggle.dispatchEvent({ type: "click" }); // -> system (back to light)
  assert.equal(root.classList.contains("dark"), false);
  assert.equal(toggle.getAttribute("aria-label"), "主题：系统");
});

test("system mode follows OS scheme changes", () => {
  const { root, mq, toggle, state } = makeEnv({ stored: "system", systemDark: false });
  assert.equal(root.classList.contains("dark"), false);
  state.dark = true; // OS switches to dark
  mq.dispatchEvent({ type: "change" });
  assert.equal(root.classList.contains("dark"), true);
  toggle.dispatchEvent({ type: "click" }); // -> light, stop following
  state.dark = false;
  mq.dispatchEvent({ type: "change" });
  assert.equal(root.classList.contains("dark"), false);
});

test("storage throwing does not break switching", () => {
  const { root, toggle } = makeEnv({ stored: null, systemDark: false, storageThrows: true });
  toggle.dispatchEvent({ type: "click" });
  toggle.dispatchEvent({ type: "click" });
  assert.equal(root.classList.contains("dark"), true);
});
