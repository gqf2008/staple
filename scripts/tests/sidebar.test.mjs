// Sidebar collapse behavior tests (issue #237).
//
// Runs the real crates/app/src/ui/sidebar.js in a Node vm with a minimal DOM
// stub (no jsdom / no node_modules), same pattern as command_palette.test.mjs.
//
//   node --test scripts/tests/*.test.mjs   # or: make js-test

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const SIDEBAR_SOURCE = readFileSync(
  new URL("../../crates/app/src/ui/sidebar.js", import.meta.url),
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
  constructor(tagName) {
    this.tagName = String(tagName).toUpperCase();
    this._attrs = new Map();
    this._classes = new Set();
    this._listeners = new Map();
    this._text = "";
    this.classList = new ClassList(this);
    this.dataset = {};
  }
  setAttribute(name, value) {
    const v = String(value);
    this._attrs.set(name, v);
    if (name === "class") this._classes = new Set(v.split(/\s+/).filter(Boolean));
  }
  getAttribute(name) { return this._attrs.has(name) ? this._attrs.get(name) : null; }
  removeAttribute(name) { this._attrs.delete(name); }
  get textContent() { return this._text; }
  set textContent(value) { this._text = String(value); }
  addEventListener(type, cb) {
    if (!this._listeners.has(type)) this._listeners.set(type, []);
    this._listeners.get(type).push(cb);
  }
  dispatchEvent(event) {
    for (const cb of this._listeners.get(event.type) || []) cb(event);
  }
}

function makeEnv({ collapsedStored = false, storageThrows = false } = {}) {
  const sidebar = new Element("nav");
  sidebar.setAttribute("class", "app-sidebar");
  sidebar.setAttribute("data-collapsible", "true");
  const toggle = new Element("button");
  toggle.setAttribute("id", "sidebar-toggle");
  toggle.setAttribute("aria-expanded", "true");
  toggle.setAttribute("aria-label", "收起侧栏");
  toggle.textContent = "«";
  toggle.dataset.collapse = "收起侧栏";
  toggle.dataset.expand = "展开侧栏";
  const document = {
    _listeners: new Map(),
    _byId: new Map([["sidebar-toggle", toggle]]),
    addEventListener(type, cb) {
      if (!this._listeners.has(type)) this._listeners.set(type, []);
      this._listeners.get(type).push(cb);
    },
    dispatchEvent(event) {
      for (const cb of this._listeners.get(event.type) || []) cb(event);
    },
    querySelector(selector) {
      if (selector === ".app-sidebar[data-collapsible]") return sidebar;
      return null;
    },
    getElementById(id) { return this._byId.get(id) || null; },
  };
  const storage = {
    getItem() {
      if (storageThrows) throw new Error("denied");
      return collapsedStored ? "1" : "0";
    },
    setItem() {
      if (storageThrows) throw new Error("denied");
    },
  };
  const context = vm.createContext({ document, localStorage: storage });
  vm.runInContext(SIDEBAR_SOURCE, context);
  document.dispatchEvent({ type: "DOMContentLoaded" });
  return { sidebar, toggle };
}

test("sidebar starts expanded without stored collapse", () => {
  const { sidebar, toggle } = makeEnv();
  assert.equal(sidebar.classList.contains("collapsed"), false);
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  assert.equal(toggle.textContent, "«");
});

test("click toggles collapsed class, aria-expanded and label", () => {
  const { sidebar, toggle } = makeEnv();
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("collapsed"), true);
  assert.equal(toggle.getAttribute("aria-expanded"), "false");
  assert.equal(toggle.getAttribute("aria-label"), "展开侧栏");
  assert.equal(toggle.textContent, "»");
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("collapsed"), false);
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  assert.equal(toggle.getAttribute("aria-label"), "收起侧栏");
});

test("stored collapse is restored on load with localized label", () => {
  const { sidebar, toggle } = makeEnv({ collapsedStored: true });
  assert.equal(sidebar.classList.contains("collapsed"), true);
  assert.equal(toggle.getAttribute("aria-expanded"), "false");
  assert.equal(toggle.getAttribute("aria-label"), "展开侧栏");
});

test("storage throwing does not break collapse", () => {
  const { sidebar, toggle } = makeEnv({ storageThrows: true });
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("collapsed"), true);
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("collapsed"), false);
});

test("no-op without sidebar markup", () => {
  const document = {
    _listeners: new Map(),
    addEventListener(type, cb) { if (!this._listeners.has(type)) this._listeners.set(type, []); this._listeners.get(type).push(cb); },
    dispatchEvent(event) { for (const cb of this._listeners.get(event.type) || []) cb(event); },
    querySelector() { return null; },
    getElementById() { return null; },
  };
  const context = vm.createContext({ document, localStorage: { getItem() { return null; }, setItem() {} } });
  vm.runInContext(SIDEBAR_SOURCE, context);
  document.dispatchEvent({ type: "DOMContentLoaded" }); // must not throw
});
