// Command palette behavior tests (issue #229).
//
// Runs the real crates/app/src/ui/command_palette.js in a Node vm with a
// minimal DOM stub (no jsdom / no node_modules). The stub mirrors only the
// DOM surface the script touches, so the suite has zero dependencies and runs
// with the Node built-in test runner:
//
//   node --test scripts/tests/*.test.mjs   # or: make js-test
//
// The fixture below mirrors the palette markup injected by
// crates/app/src/ui/layout.rs.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const PALETTE_SOURCE = readFileSync(
  new URL("../../crates/app/src/ui/command_palette.js", import.meta.url),
  "utf8",
);

// --- Minimal DOM stub ----------------------------------------------------

class ClassList {
  constructor(el) {
    this._el = el;
  }
  toggle(name, force) {
    const classes = this._el._classes;
    const has = classes.has(name);
    const should = force === undefined ? !has : Boolean(force);
    if (should && !has) classes.add(name);
    else if (!should && has) classes.delete(name);
    return should;
  }
}

class Element {
  constructor(tagName) {
    this.tagName = String(tagName).toUpperCase();
    this.children = [];
    this.parentNode = null;
    this._attrs = new Map();
    this._classes = new Set();
    this._listeners = new Map();
    this._text = "";
    this.value = "";
    this._hidden = false;
    this.classList = new ClassList(this);
  }

  setAttribute(name, value) {
    const v = String(value);
    this._attrs.set(name, v);
    if (name === "class") this._classes = new Set(v.split(/\s+/).filter(Boolean));
    if (name === "hidden") this._hidden = v !== "false" && v !== "";
  }

  getAttribute(name) {
    return this._attrs.has(name) ? this._attrs.get(name) : null;
  }

  removeAttribute(name) {
    this._attrs.delete(name);
    if (name === "class") this._classes.clear();
    if (name === "hidden") this._hidden = false;
  }

  get hidden() {
    return this._hidden;
  }

  set hidden(value) {
    this._hidden = Boolean(value);
    if (this._hidden) this._attrs.set("hidden", "hidden");
    else this._attrs.delete("hidden");
  }

  get className() {
    return this.getAttribute("class") || "";
  }

  set className(value) {
    this.setAttribute("class", value);
  }

  get textContent() {
    if (this.tagName === "#TEXT") return this._text;
    return this._text + this.children.map((child) => child.textContent).join("");
  }

  set textContent(value) {
    this._text = String(value);
    this.children = [];
  }

  appendChild(child) {
    if (child.parentNode) child.remove();
    child.parentNode = this;
    this.children.push(child);
    return child;
  }

  remove() {
    if (this.parentNode) {
      const siblings = this.parentNode.children;
      const index = siblings.indexOf(this);
      if (index !== -1) siblings.splice(index, 1);
      this.parentNode = null;
    }
  }

  addEventListener(type, callback) {
    if (!this._listeners.has(type)) this._listeners.set(type, []);
    this._listeners.get(type).push(callback);
  }

  dispatchEvent(event) {
    for (const callback of this._listeners.get(event.type) || []) callback(event);
  }

  focus() {}
  scrollIntoView() {}

  matches(selector) {
    if (selector.startsWith("#")) return this.getAttribute("id") === selector.slice(1);
    if (selector.startsWith(".")) {
      return (this.getAttribute("class") || "")
        .split(/\s+/)
        .includes(selector.slice(1));
    }
    return this.tagName.toLowerCase() === selector.toLowerCase();
  }

  querySelector(selector) {
    return this._find(selector, false);
  }

  querySelectorAll(selector) {
    return this._find(selector, true);
  }

  _find(selector, all) {
    const out = all ? [] : null;
    const walk = (node) => {
      for (const child of node.children) {
        if (child.matches(selector)) {
          if (all) out.push(child);
          else return child;
        }
        const nested = walk(child);
        if (nested) return nested;
      }
      return null;
    };
    const found = walk(this);
    return all ? out : found;
  }
}

class TextNode extends Element {
  constructor(text) {
    super("#text");
    this._text = String(text);
  }
}

function createDocument() {
  return {
    _listeners: new Map(),
    _byId: new Map(),
    addEventListener(type, callback) {
      if (!this._listeners.has(type)) this._listeners.set(type, []);
      this._listeners.get(type).push(callback);
    },
    dispatchEvent(event) {
      for (const callback of this._listeners.get(event.type) || []) callback(event);
    },
    getElementById(id) {
      return this._byId.get(id) || null;
    },
    createElement(tagName) {
      return new Element(tagName);
    },
    createTextNode(text) {
      return new TextNode(text);
    },
  };
}

function registerIds(document, root) {
  const walk = (node) => {
    const id = node.getAttribute("id");
    if (id) document._byId.set(id, node);
    node.children.forEach(walk);
  };
  walk(root);
}

function walkAll(root) {
  const out = [];
  const walk = (node) => {
    for (const child of node.children) {
      out.push(child);
      walk(child);
    }
  };
  walk(root);
  return out;
}

// --- Fixture -------------------------------------------------------------

function text(value) {
  return new TextNode(value);
}

function el(tagName, attrs = {}, children = []) {
  const node = new Element(tagName);
  for (const [name, value] of Object.entries(attrs)) {
    if (name === "hidden" && value) node.hidden = true;
    else node.setAttribute(name, value);
  }
  for (const child of children) node.appendChild(child);
  return node;
}

// Mirrors the palette markup in crates/app/src/ui/layout.rs (English labels).
function buildPaletteDOM(companyId) {
  const staticItems = [
    el("a", { class: "command-item", href: "/" }, [text("Companies")]),
  ];
  if (companyId) {
    staticItems.push(
      el("a", { class: "command-item", href: `/companies/${companyId}/board` }, [text("Board")]),
      el("a", { class: "command-item", href: `/companies/${companyId}/issues` }, [text("Issues")]),
      el("a", { class: "command-item", href: `/companies/${companyId}/settings` }, [text("Settings")]),
    );
  }
  staticItems.push(
    el("a", { class: "command-item", href: "/instance/settings" }, [text("Instance settings")]),
    el("a", { class: "command-item", href: "/profile/settings" }, [text("Profile settings")]),
    el("a", { class: "command-item", href: "/adapters" }, [text("Adapters")]),
  );

  const list = el("div", { id: "command-list", class: "command-palette-list" }, [
    ...staticItems,
    el("div", { id: "command-empty", class: "command-empty", hidden: true, role: "status" }, [
      text("No matches."),
    ]),
  ]);
  const panel = el("div", { class: "command-palette-panel", "data-company-id": companyId || "" }, [
    el("input", { id: "command-input", class: "command-palette-input", type: "text" }),
    list,
  ]);
  const palette = el(
    "div",
    { id: "command-palette", class: "command-palette", hidden: true, role: "dialog", "aria-modal": "true" },
    [panel],
  );
  return { palette, panel, list, input: null, empty: null, staticItems };
}

function loadPalette({ companyId = "company-1" } = {}) {
  const document = createDocument();
  const { palette, panel, list, staticItems } = buildPaletteDOM(companyId);
  registerIds(document, palette);
  const input = document.getElementById("command-input");
  const empty = document.getElementById("command-empty");

  const state = { fetchCalls: [], resolvers: [] };
  const fetchMock = (url) =>
    new Promise((resolve) => {
      state.fetchCalls.push(url);
      state.resolvers.push(resolve);
    });

  const window = { location: { search: "?lang=en", href: "http://staple.test/companies/company-1/board" } };
  const context = vm.createContext({
    document,
    window,
    fetch: fetchMock,
    URLSearchParams,
    setTimeout,
    clearTimeout,
    console,
  });
  vm.runInContext(PALETTE_SOURCE, context, { filename: "command_palette.js" });

  document.dispatchEvent({ type: "DOMContentLoaded" });

  return { document, window, palette, panel, input, list, empty, staticItems, state };
}

// --- Event helpers -------------------------------------------------------

function keydown(target, key, options = {}) {
  const event = {
    type: "keydown",
    key,
    metaKey: false,
    ctrlKey: false,
    defaultPrevented: false,
    preventDefault() {
      this.defaultPrevented = true;
    },
    ...options,
  };
  target.dispatchEvent(event);
  return event;
}

function typeInput(input, value) {
  input.value = value;
  input.dispatchEvent({ type: "input" });
}

function wait(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function flush() {
  await wait(0);
  await wait(0);
}

function activeIdCount(handle) {
  return walkAll(handle.palette).filter((node) => node.getAttribute("id") === "command-active-item").length;
}

function activeItem(handle) {
  return walkAll(handle.palette).find((node) => node.getAttribute("id") === "command-active-item") || null;
}

function issueItems(handle) {
  return walkAll(handle.palette).filter((node) =>
    (node.getAttribute("href") || "").startsWith("/issues/"),
  );
}

function visibleTexts(handle) {
  return handle.staticItems.filter((item) => !item.hidden).map((item) => item.textContent);
}

// --- Tests ---


test("Cmd/Ctrl+K toggles the palette open and closed", () => {
  const handle = loadPalette();
  assert.equal(handle.palette.hidden, true, "palette starts hidden");

  keydown(handle.document, "k", { metaKey: true });
  assert.equal(handle.palette.hidden, false, "Cmd+K opens the palette");
  assert.equal(handle.input.value, "", "open resets the query");

  keydown(handle.document, "k", { ctrlKey: true });
  assert.equal(handle.palette.hidden, true, "Ctrl+K closes the palette");

  keydown(handle.document, "K", { metaKey: true });
  assert.equal(handle.palette.hidden, false, "Cmd+Shift+K still toggles (lowercased key)");
});

test("open resets filter, selection and issue items", async () => {
  const handle = loadPalette();
  keydown(handle.document, "k", { metaKey: true });
  typeInput(handle.input, "boa");
  keydown(handle.input, "ArrowDown");
  assert.equal(activeItem(handle).textContent, "Board");

  keydown(handle.document, "k", { metaKey: true }); // close
  keydown(handle.document, "k", { metaKey: true }); // reopen
  assert.equal(handle.input.value, "", "reopen clears the query");
  assert.equal(activeIdCount(handle), 0, "reopen clears the selection");
  assert.deepEqual(visibleTexts(handle), [
    "Companies",
    "Board",
    "Issues",
    "Settings",
    "Instance settings",
    "Profile settings",
    "Adapters",
  ]);
  assert.equal(issueItems(handle).length, 0, "reopen drops fetched issue items");
});

test("ArrowDown/ArrowUp select with wrapping and keep a single active id", () => {
  const handle = loadPalette();
  keydown(handle.document, "k", { metaKey: true });
  const labels = visibleTexts(handle);
  const last = labels.length - 1;

  // ArrowDown from -1 selects the first item.
  keydown(handle.input, "ArrowDown");
  assert.equal(activeItem(handle).textContent, labels[0]);
  assert.equal(handle.input.getAttribute("aria-activedescendant"), "command-active-item");
  assert.equal(activeIdCount(handle), 1, "#command-active-item must be unique");

  // ArrowDown moves forward and clamps at the last item.
  keydown(handle.input, "ArrowDown");
  keydown(handle.input, "ArrowDown");
  assert.equal(activeItem(handle).textContent, labels[2]);
  for (let i = 0; i < labels.length; i += 1) keydown(handle.input, "ArrowDown");
  assert.equal(activeItem(handle).textContent, labels[last], "ArrowDown clamps at the last item");
  assert.equal(activeIdCount(handle), 1);

  // ArrowUp moves backward.
  keydown(handle.input, "ArrowUp");
  assert.equal(activeItem(handle).textContent, labels[last - 1]);

  // ArrowUp at the first item wraps to the last.
  // From last-1, pressing ArrowUp labels.length - 2 times lands on the first item.
  for (let i = 0; i < labels.length - 2; i += 1) keydown(handle.input, "ArrowUp");
  assert.equal(activeItem(handle).textContent, labels[0]);
  keydown(handle.input, "ArrowUp");
  assert.equal(activeItem(handle).textContent, labels[last], "ArrowUp wraps to the last item");
  assert.equal(activeIdCount(handle), 1, "wrapping keeps #command-active-item unique");
});

test("Enter navigates to the active item href and no-ops without selection", () => {
  const handle = loadPalette();
  const originalHref = handle.window.location.href;
  keydown(handle.document, "k", { metaKey: true });

  // No selection: Enter must not navigate.
  const noop = keydown(handle.input, "Enter");
  assert.equal(handle.window.location.href, originalHref);
  assert.equal(noop.defaultPrevented, true);

  keydown(handle.input, "ArrowDown");
  keydown(handle.input, "ArrowDown");
  const enter = keydown(handle.input, "Enter");
  assert.equal(enter.defaultPrevented, true);
  assert.equal(handle.window.location.href, "/companies/company-1/board");
});

test("Escape closes the palette", () => {
  const handle = loadPalette();
  keydown(handle.document, "k", { metaKey: true });
  assert.equal(handle.palette.hidden, false);
  keydown(handle.input, "Escape");
  assert.equal(handle.palette.hidden, true);
});

test("input filters static items and drives the empty state", () => {
  const handle = loadPalette({ companyId: "" });

  keydown(handle.document, "k", { metaKey: true });
  assert.equal(handle.empty.hidden, true, "empty is hidden while items match");

  typeInput(handle.input, "inst");
  assert.deepEqual(visibleTexts(handle), ["Instance settings"]);
  assert.equal(handle.empty.hidden, true, "empty stays hidden when at least one item matches");

  typeInput(handle.input, "zzzz-no-match");
  assert.deepEqual(visibleTexts(handle), []);
  assert.equal(handle.empty.hidden, false, "empty becomes visible when nothing matches");
  assert.equal(activeIdCount(handle), 0, "filtering clears the previous selection");
  assert.equal(handle.input.getAttribute("aria-activedescendant"), null);

  typeInput(handle.input, "");
  assert.equal(visibleTexts(handle).length, handle.staticItems.length, "clearing restores all items");
  assert.equal(handle.empty.hidden, true);
});

test("fetched issues are filtered, capped at 50 and hidden once empty", async () => {
  const handle = loadPalette();
  keydown(handle.document, "k", { metaKey: true });

  typeInput(handle.input, "alpha");
  await wait(250);
  assert.equal(handle.state.fetchCalls.length, 1);
  assert.equal(handle.state.fetchCalls[0], "/api/companies/company-1/issues");

  const issues = Array.from({ length: 60 }, (_, i) => ({
    id: `issue-${i + 1}`,
    identifier: `ALPHA-${i + 1}`,
    title: `Alpha issue ${i + 1}`,
  }));
  handle.state.resolvers[0]({ json: async () => issues });
  await flush();

  const appended = issueItems(handle);
  assert.equal(appended.length, 50, "issue results are capped at 50");
  assert.equal(appended[0].getAttribute("href"), "/issues/issue-1?lang=en");
  assert.equal(appended[49].getAttribute("href"), "/issues/issue-50?lang=en");
  assert.equal(
    appended.some((item) => item.getAttribute("href") === "/issues/issue-51?lang=en"),
    false,
    "items beyond the cap are dropped",
  );
  assert.equal(handle.empty.hidden, true, "empty stays hidden while capped results match");

  typeInput(handle.input, "nomatch");
  await wait(250);
  handle.state.resolvers[1]({ json: async () => [] });
  await flush();
  assert.equal(issueItems(handle).length, 0, "a non-matching query drops previous issue items");
  assert.equal(handle.empty.hidden, false, "empty is visible when the query matches nothing");
});

test("a stale (late) response is discarded when a newer query wins", async () => {
  const handle = loadPalette();
  keydown(handle.document, "k", { metaKey: true });

  typeInput(handle.input, "a");
  await wait(250);
  typeInput(handle.input, "ab");
  await wait(250);
  assert.equal(handle.state.fetchCalls.length, 2);

  // Newer query resolves first; its items are shown.
  handle.state.resolvers[1]({
    json: async () => [{ id: "ab-1", identifier: "AB-1", title: "ab issue" }],
  });
  await flush();
  assert.deepEqual(
    issueItems(handle).map((item) => item.getAttribute("href")),
    ["/issues/ab-1?lang=en"],
  );

  // The stale response arrives later and must be dropped (request token mismatch).
  handle.state.resolvers[0]({
    json: async () => [{ id: "a-1", identifier: "A-1", title: "a issue" }],
  });
  await flush();
  assert.deepEqual(
    issueItems(handle).map((item) => item.getAttribute("href")),
    ["/issues/ab-1?lang=en"],
    "the older response must not overwrite the newer results",
  );
});

test("closing the palette discards a pending fetch response", async () => {
  const handle = loadPalette();
  keydown(handle.document, "k", { metaKey: true });

  typeInput(handle.input, "late");
  await wait(250);
  assert.equal(handle.state.fetchCalls.length, 1);

  keydown(handle.input, "Escape");
  assert.equal(handle.palette.hidden, true);

  handle.state.resolvers[0]({
    json: async () => [{ id: "late-1", identifier: "LATE-1", title: "late issue" }],
  });
  await flush();
  assert.equal(issueItems(handle).length, 0, "results must not be injected into a closed palette");
});

