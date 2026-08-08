// Issues toolbar behavior tests (issue #268): "+ New Task" toggles the inline
// quick-create form, and the search input filters the issue list.
// Runs the real crates/app/src/ui/issues.js in a Node vm with a minimal DOM
// stub (same pattern as ui_feedback.test.mjs). Zero dependencies.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const SOURCE = readFileSync(
  new URL("../../crates/app/src/ui/issues.js", import.meta.url),
  "utf8",
);

class Element {
  constructor(tagName) {
    this.tagName = tagName.toUpperCase();
    this.children = [];
    this._attrs = new Map();
    this._listeners = new Map();
    this.hidden = false;
    this.value = "";
    this.textContent = "";
    this.focused = false;
  }
  setAttribute(name, value) { this._attrs.set(String(name), String(value)); }
  getAttribute(name) { return this._attrs.has(String(name)) ? this._attrs.get(String(name)) : null; }
  addEventListener(type, fn) { (this._listeners.get(type) || this._listeners.set(type, []).get(type)).push(fn); }
  dispatch(type) { for (const fn of this._listeners.get(type) || []) fn.call(this, { target: this }); }
  focus() { this.focused = true; }
  querySelectorAll() { return []; }
  querySelector() { return null; }
}

class List extends Element {
  constructor() {
    super("UL");
    this.rows = [];
  }
  querySelectorAll(sel) {
    if (sel === "li" || sel === "li:not(#issue-empty)") {
      if (sel.includes(":not(#issue-empty)")) return this.rows.filter((r) => r !== this.empty);
      return this.rows;
    }
    return [];
  }
  querySelector(sel) {
    if (sel === "#issue-empty") return this.empty;
    return null;
  }
}

function makeDoc() {
  const toggle = new Element("BUTTON");
  const form = new Element("FORM");
  form.hidden = true;
  const title = new Element("INPUT");
  const search = new Element("INPUT");
  search.value = "";
  const list = new List();
  const row1 = new Element("LI"); row1.setAttribute("data-search", "ACM-1 Hire"); row1.textContent = "ACM-1 Hire";
  const row2 = new Element("LI"); row2.setAttribute("data-search", "ACM-2 QA invite"); row2.textContent = "ACM-2 QA invite";
  list.empty = new Element("LI"); list.empty.hidden = true;
  list.empty.textContent = "No matches.";
  list.rows = [row1, row2, list.empty];
  const byId = {
    "new-issue-toggle": toggle,
    "new-issue-form": form,
    "new-issue-title": title,
    "issue-search": search,
    "issue-list": list,
    "issue-empty": list.empty,
  };
  const doc = {
    getElementById: (id) => byId[id] || null,
    addEventListener: (type, fn) => { doc._ready = fn; },
  };
  return { doc, toggle, form, title, search, list, row1, row2 };
}

function run(doc) {
  vm.runInNewContext(SOURCE, { document: doc });
  if (doc._ready) doc._ready();
}

test("issues.js toggles the quick-create form and focuses the title input", () => {
  const { doc, toggle, form, title } = makeDoc();
  run(doc);
  assert.equal(form.hidden, true);
  toggle.dispatch("click");
  assert.equal(form.hidden, false);
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  assert.equal(title.focused, true);
  toggle.dispatch("click");
  assert.equal(form.hidden, true);
  assert.equal(toggle.getAttribute("aria-expanded"), "false");
});

test("issues.js filters rows by search term and toggles the empty state", () => {
  const { doc, search, list, row1, row2 } = makeDoc();
  run(doc);
  search.value = "qa";
  search.dispatch("input");
  assert.equal(row1.hidden, true);
  assert.equal(row2.hidden, false);
  assert.equal(list.empty.hidden, true);

  search.value = "zzz-no-match";
  search.dispatch("input");
  assert.equal(row1.hidden, true);
  assert.equal(row2.hidden, true);
  assert.equal(list.empty.hidden, false);

  search.value = "";
  search.dispatch("input");
  assert.equal(row1.hidden, false);
  assert.equal(row2.hidden, false);
});
