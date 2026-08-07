// Sidebar collapse + resizable-width behavior tests (issue #237 + #244).
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
    this.style = {};
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
    const ev = event && typeof event.preventDefault === "function"
      ? event
      : Object.assign({}, event, { preventDefault() {} });
    for (const cb of this._listeners.get(ev.type) || []) cb(ev);
  }
  setPointerCapture() {}
  preventDefault() {}
}

function makeEnv({
  collapsedStored = false,
  widthStored = null,
  storageThrows = false,
} = {}) {
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
  const resizer = new Element("div");
  resizer.setAttribute("id", "sidebar-resizer");
  const windowObj = new Element("window");
  const document = {
    _listeners: new Map(),
    _byId: new Map([["sidebar-toggle", toggle], ["sidebar-resizer", resizer]]),
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
    getItem(key) {
      if (storageThrows) throw new Error("denied");
      if (key === "staple.sidebar.collapsed") return collapsedStored ? "1" : "0";
      if (key === "staple.sidebar.width") return widthStored;
      return null;
    },
    setItem() {
      if (storageThrows) throw new Error("denied");
    },
  };
  const context = vm.createContext({ document, window: windowObj, localStorage: storage });
  vm.runInContext(SIDEBAR_SOURCE, context);
  document.dispatchEvent({ type: "DOMContentLoaded" });
  return { sidebar, toggle, resizer, windowObj, storage };
}

test("sidebar starts expanded with default 240px width", () => {
  const { sidebar, toggle } = makeEnv();
  assert.equal(sidebar.classList.contains("collapsed"), false);
  assert.equal(sidebar.style.width, "240px");
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  assert.equal(toggle.textContent, "«");
});

test("click toggles collapsed class, aria-expanded and label", () => {
  const { sidebar, toggle } = makeEnv();
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("collapsed"), true);
  assert.equal(sidebar.style.width, "", "collapsed uses class width");
  assert.equal(toggle.getAttribute("aria-expanded"), "false");
  assert.equal(toggle.getAttribute("aria-label"), "展开侧栏");
  assert.equal(toggle.textContent, "»");
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("collapsed"), false);
  assert.equal(sidebar.style.width, "240px");
});

test("stored collapse is restored on load with localized label", () => {
  const { sidebar, toggle } = makeEnv({ collapsedStored: true });
  assert.equal(sidebar.classList.contains("collapsed"), true);
  assert.equal(sidebar.style.width, "");
  assert.equal(toggle.getAttribute("aria-label"), "展开侧栏");
});

test("stored width is clamped to 208-420 on load", () => {
  assert.equal(makeEnv({ widthStored: "500" }).sidebar.style.width, "420px");
  assert.equal(makeEnv({ widthStored: "100" }).sidebar.style.width, "208px");
  assert.equal(makeEnv({ widthStored: "320" }).sidebar.style.width, "320px");
});

test("dragging resizer updates and persists width", () => {
  const { sidebar, resizer, windowObj, storage } = makeEnv({ widthStored: "240" });
  resizer.dispatchEvent({ type: "pointerdown", clientX: 100, pointerId: 1 });
  windowObj.dispatchEvent({ type: "pointermove", clientX: 150 });
  assert.equal(sidebar.style.width, "290px");
  windowObj.dispatchEvent({ type: "pointerup" });
  assert.equal(sidebar.style.width, "290px");
  let saved = null;
  const orig = storage.setItem;
  storage.setItem = function (key, value) { if (key === "staple.sidebar.width") saved = value; orig.call(this, key, value); };
  // second drag persists
  resizer.dispatchEvent({ type: "pointerdown", clientX: 0, pointerId: 2 });
  windowObj.dispatchEvent({ type: "pointermove", clientX: -200 });
  assert.equal(sidebar.style.width, "208px");
  windowObj.dispatchEvent({ type: "pointerup" });
  assert.equal(saved, "208");
});

test("dragging is ignored while collapsed", () => {
  const { sidebar, resizer, windowObj } = makeEnv({ collapsedStored: true });
  resizer.dispatchEvent({ type: "pointerdown", clientX: 100, pointerId: 1 });
  windowObj.dispatchEvent({ type: "pointermove", clientX: 300 });
  assert.equal(sidebar.style.width, "", "collapsed width unchanged");
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
  const windowObj = new Element("window");
  const context = vm.createContext({ document, window: windowObj, localStorage: { getItem() { return null; }, setItem() {} } });
  vm.runInContext(SIDEBAR_SOURCE, context);
  document.dispatchEvent({ type: "DOMContentLoaded" }); // must not throw
});
