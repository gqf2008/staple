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
  remove(name) { this._el._classes.delete(name); }
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
  narrow = false,
  withResizer = true,
  mobileOpenStored = false,
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
  const scrim = new Element("div");
  scrim.setAttribute("id", "sidebar-scrim");
  scrim.hidden = true;
  const windowObj = new Element("window");
  const mediaState = { matches: narrow };
  const mediaListeners = [];
  windowObj.matchMedia = () => ({
    get matches() { return mediaState.matches; },
    addEventListener(type, cb) { mediaListeners.push(cb); },
    addListener(cb) { mediaListeners.push(cb); },
  });
  function setNarrow(value) {
    mediaState.matches = value;
    for (const cb of mediaListeners) cb();
  }
  const document = {
    _listeners: new Map(),
    _byId: new Map(withResizer
      ? [["sidebar-toggle", toggle], ["sidebar-resizer", resizer], ["sidebar-scrim", scrim]]
      : [["sidebar-toggle", toggle], ["sidebar-scrim", scrim]]),
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
      if (key === "staple.sidebar.mobileOpen") return mobileOpenStored ? "1" : "0";
      return null;
    },
    setItem() {
      if (storageThrows) throw new Error("denied");
    },
  };
  const context = vm.createContext({ document, window: windowObj, localStorage: storage });
  vm.runInContext(SIDEBAR_SOURCE, context);
  document.dispatchEvent({ type: "DOMContentLoaded" });
  return { sidebar, toggle, resizer, scrim, windowObj, storage, document, setNarrow };
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

test("drag disables width transition while dragging and restores after", () => {
  const { sidebar, resizer, windowObj } = makeEnv({ widthStored: "240" });
  resizer.dispatchEvent({ type: "pointerdown", clientX: 100, pointerId: 1 });
  assert.equal(sidebar.style.transition, "none");
  windowObj.dispatchEvent({ type: "pointermove", clientX: 200 });
  assert.equal(sidebar.style.width, "340px");
  windowObj.dispatchEvent({ type: "pointerup" });
  assert.equal(sidebar.style.transition, "");
});

test("pointercancel restores start width and transition", () => {
  const { sidebar, resizer, windowObj } = makeEnv({ widthStored: "240" });
  resizer.dispatchEvent({ type: "pointerdown", clientX: 100, pointerId: 1 });
  windowObj.dispatchEvent({ type: "pointermove", clientX: 300 });
  assert.equal(sidebar.style.width, "420px"); // clamped at MAX
  windowObj.dispatchEvent({ type: "pointercancel" });
  assert.equal(sidebar.style.width, "240px");
  assert.equal(sidebar.style.transition, "");
});

test("drag is ignored on narrow screens", () => {
  const { sidebar, resizer, windowObj } = makeEnv({ widthStored: "240", narrow: true });
  resizer.dispatchEvent({ type: "pointerdown", clientX: 100, pointerId: 1 });
  windowObj.dispatchEvent({ type: "pointermove", clientX: 300 });
  assert.equal(sidebar.style.width, "240px");
  assert.equal(sidebar.style.transition, undefined);
});

test("keyboard resizing: ArrowRight/Left/Home/End", () => {
  const { sidebar, resizer, storage } = makeEnv({ widthStored: "240" });
  let saved = null;
  const orig = storage.setItem;
  storage.setItem = function (key, value) { if (key === "staple.sidebar.width") saved = value; orig.call(this, key, value); };
  resizer.dispatchEvent({ type: "keydown", key: "ArrowRight" });
  assert.equal(sidebar.style.width, "256px");
  assert.equal(resizer.getAttribute("aria-valuenow"), "256");
  assert.equal(saved, "256");
  resizer.dispatchEvent({ type: "keydown", key: "ArrowLeft" });
  assert.equal(sidebar.style.width, "240px");
  resizer.dispatchEvent({ type: "keydown", key: "End" });
  assert.equal(sidebar.style.width, "420px");
  resizer.dispatchEvent({ type: "keydown", key: "Home" });
  assert.equal(sidebar.style.width, "208px");
});

test("invalid stored widths fall back to defaults / clamp", () => {
  assert.equal(makeEnv({ widthStored: "abc" }).sidebar.style.width, "240px");
  assert.equal(makeEnv({ widthStored: "240.6" }).sidebar.style.width, "241px");
  assert.equal(makeEnv({ widthStored: "" }).sidebar.style.width, "240px");
});

test("no-op without resizer element", () => {
  const env = makeEnv({ widthStored: "320", withResizer: false });
  assert.equal(env.sidebar.style.width, "320px"); // collapse still works
  env.toggle.dispatchEvent({ type: "click" });
  assert.equal(env.sidebar.classList.contains("collapsed"), true);
});

test("narrow mode: drawer starts closed with hamburger and hidden scrim", () => {
  const { sidebar, toggle, scrim } = makeEnv({ narrow: true });
  assert.equal(sidebar.classList.contains("drawer-open"), false);
  assert.equal(scrim.hidden, true);
  assert.equal(toggle.textContent, "☰");
  assert.equal(toggle.getAttribute("aria-expanded"), "false");
});

test("narrow mode: toggle opens drawer with scrim, closes on scrim click", () => {
  const { sidebar, toggle, scrim } = makeEnv({ narrow: true });
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("drawer-open"), true);
  assert.equal(scrim.hidden, false);
  assert.equal(toggle.textContent, "×");
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  scrim.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("drawer-open"), false);
  assert.equal(scrim.hidden, true);
});

test("narrow mode: Escape closes the drawer", () => {
  const { sidebar, toggle, document } = makeEnv({ narrow: true });
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("drawer-open"), true);
  document.dispatchEvent({ type: "keydown", key: "Escape" });
  assert.equal(sidebar.classList.contains("drawer-open"), false);
});

test("narrow mode: stored open state is restored", () => {
  const env = makeEnv({ narrow: true, mobileOpenStored: true });
  assert.equal(env.sidebar.classList.contains("drawer-open"), true);
  assert.equal(env.scrim.hidden, false);
});

test("narrow mode: desktop collapse/width persistence is ignored", () => {
  const { sidebar } = makeEnv({ narrow: true, collapsedStored: true, widthStored: "320" });
  assert.equal(sidebar.classList.contains("collapsed"), false);
  assert.equal(sidebar.style.width, "240px");
});

test("cross-breakpoint: desktop -> narrow -> toggle opens drawer", () => {
  const { sidebar, toggle, setNarrow } = makeEnv({ narrow: false });
  setNarrow(true);
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("drawer-open"), true);
  assert.equal(sidebar.classList.contains("collapsed"), false);
});

test("cross-breakpoint: narrow -> desktop -> toggle collapses", () => {
  const { sidebar, toggle, setNarrow } = makeEnv({ narrow: true });
  setNarrow(false);
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("collapsed"), true);
  assert.equal(sidebar.classList.contains("drawer-open"), false);
});

test("cross-breakpoint: narrow open -> desktop resets drawer state (Esc no-op)", () => {
  const { sidebar, toggle, setNarrow, document } = makeEnv({ narrow: true });
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.classList.contains("drawer-open"), true);
  setNarrow(false);
  assert.equal(sidebar.classList.contains("drawer-open"), false);
  assert.equal(sidebar.inert, false);
  document.dispatchEvent({ type: "keydown", key: "Escape" });
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
});

test("desktop toggle width follows collapsed rail", () => {
  const { sidebar, toggle } = makeEnv({ collapsedStored: true });
  assert.equal(sidebar.classList.contains("collapsed"), true);
  assert.equal(toggle.style.width, "calc(var(--sidebar-rail) - var(--space-6))");
});

test("desktop toggle width follows expanded width", () => {
  const { toggle } = makeEnv({ widthStored: "320" });
  assert.equal(toggle.style.width, "calc(320px - var(--space-6))");
});

test("drawer closed sets sidebar inert, open removes it", () => {
  const { sidebar, toggle } = makeEnv({ narrow: true });
  assert.equal(sidebar.inert, true);
  toggle.dispatchEvent({ type: "click" });
  assert.equal(sidebar.inert, false);
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
