// UI feedback behavior tests (issue #231).
//
// Runs the real crates/app/src/ui/ui_feedback.js in a Node vm with a minimal
// DOM stub (no jsdom / no node_modules), mirroring the pattern used by
// scripts/tests/command_palette.test.mjs. Zero dependencies; runs with the
// Node built-in test runner:
//
//   node --test scripts/tests/*.test.mjs   # or: make js-test

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const FEEDBACK_SOURCE = readFileSync(
  new URL("../../crates/app/src/ui/ui_feedback.js", import.meta.url),
  "utf8",
);

// --- Minimal DOM stub ----------------------------------------------------

class Element {
  constructor(tagName, attrs = {}) {
    this.tagName = String(tagName).toUpperCase();
    this.type = attrs.type || (this.tagName === "BUTTON" ? "submit" : "");
    this.disabled = false;
    this.value = "";
    this.children = [];
    this.parentNode = null;
    this._attrs = new Map();
    this._classes = new Set();
    this._listeners = new Map();
    this._hidden = false;
    this.classList = { add: (name) => this._classes.add(name) };
    for (const [name, value] of Object.entries(attrs)) {
      if (name === "hidden" && value) this.hidden = true;
      else this.setAttribute(name, value);
    }
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

  hasAttribute(name) {
    return this._attrs.has(name);
  }

  get className() {
    return this.getAttribute("class") || "";
  }

  set className(value) {
    this.setAttribute("class", value);
  }

  get hidden() {
    return this._hidden;
  }

  set hidden(value) {
    this._hidden = Boolean(value);
    if (this._hidden) this._attrs.set("hidden", "hidden");
    else this._attrs.delete("hidden");
  }

  appendChild(child) {
    child.parentNode = this;
    this.children.push(child);
    return child;
  }

  addEventListener(type, callback) {
    if (!this._listeners.has(type)) this._listeners.set(type, []);
    this._listeners.get(type).push(callback);
  }

  dispatchEvent(event) {
    for (const callback of this._listeners.get(event.type) || []) callback.call(this, event);
  }

  matches(selector) {
    const match = selector.match(/^([a-z]+)\[([a-z-]+)="([^"]*)"\]$/i);
    if (!match) return false;
    const [, tag, attr, value] = match;
    return (
      this.tagName.toLowerCase() === tag.toLowerCase() &&
      (this.getAttribute(attr) || "") === value
    );
  }

  querySelectorAll(selector) {
    const out = [];
    const walk = (node) => {
      for (const child of node.children) {
        if (child.matches(selector)) out.push(child);
        walk(child);
      }
    };
    walk(this);
    return out;
  }
}

function createDocument(forms, toast) {
  const document = {
    _listeners: new Map(),
    _byId: new Map(),
    activeElement: null,
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
    querySelectorAll(selector) {
      const out = [];
      const walk = (node) => {
        for (const child of node.children) {
          if (child.matches(selector)) out.push(child);
          walk(child);
        }
      };
      for (const root of [...forms, ...(toast ? [toast] : [])]) {
        if (root.matches(selector)) out.push(root);
        walk(root);
      }
      return out;
    },
  };
  for (const root of [...forms, ...(toast ? [toast] : [])]) {
    const walk = (node) => {
      const id = node.getAttribute("id");
      if (id) document._byId.set(id, node);
      node.children.forEach(walk);
    };
    walk(root);
  }
  return document;
}

function fakeTimers() {
  const timeouts = [];
  return {
    timeouts,
    setTimeout(fn) {
      timeouts.push({ fn });
      return timeouts.length;
    },
    clearTimeout() {},
    runAll() {
      while (timeouts.length) {
        const { fn } = timeouts.shift();
        fn();
      }
    },
  };
}

// --- Fixture -------------------------------------------------------------

function buildForms({ withToast = false } = {}) {
  const regular = new Element("form", { method: "post" });
  const regularButton = new Element("button", { type: "submit" });
  regular.appendChild(regularButton);

  const noFeedback = new Element("form", { method: "post", "data-no-feedback": "true" });
  const noFeedbackButton = new Element("button", { type: "submit" });
  noFeedback.appendChild(noFeedbackButton);

  const getForm = new Element("form", { method: "get" });
  const getButton = new Element("button", { type: "submit" });
  getForm.appendChild(getButton);

  const toast = withToast
    ? new Element("div", { id: "flash-toast", class: "toast toast-success" })
    : null;

  return { regular, regularButton, noFeedback, noFeedbackButton, getForm, getButton, toast };
}

function loadFeedback({ withToast = false } = {}) {
  const forms = buildForms({ withToast });
  const document = createDocument(
    [forms.regular, forms.noFeedback, forms.getForm],
    forms.toast,
  );
  const timers = fakeTimers();
  const context = vm.createContext({
    document,
    setTimeout: timers.setTimeout,
    clearTimeout: timers.clearTimeout,
    console,
  });
  vm.runInContext(FEEDBACK_SOURCE, context, { filename: "ui_feedback.js" });
  document.dispatchEvent({ type: "DOMContentLoaded" });
  return { document, timers, ...forms };
}

function submit(form, button) {
  const event = { type: "submit" };
  form.dispatchEvent(event);
  return event;
}

// --- Tests ---------------------------------------------------------------

test("submitting a mutating form disables its submit button and adds a spinner", () => {
  const handle = loadFeedback();
  handle.document.activeElement = handle.regularButton;
  submit(handle.regular, handle.regularButton);

  assert.equal(handle.regularButton.disabled, true);
  assert.ok(
    handle.regularButton._classes.has("btn-loading"),
    "button must get the btn-loading class",
  );
  const spinners = handle.regularButton.children.filter(
    (child) => child.tagName === "SPAN" && child.className === "spinner",
  );
  assert.equal(spinners.length, 1, "button must contain one spinner span");
  assert.equal(spinners[0].getAttribute("aria-hidden"), "true");
  assert.equal(handle.regular.getAttribute("aria-busy"), "true");
});

test("forms with data-no-feedback are skipped", () => {
  const handle = loadFeedback();
  handle.document.activeElement = handle.noFeedbackButton;
  submit(handle.noFeedback, handle.noFeedbackButton);

  assert.equal(handle.noFeedbackButton.disabled, false);
  assert.equal(handle.noFeedbackButton.children.length, 0);
  assert.equal(handle.noFeedback.hasAttribute("aria-busy"), false);
});

test("non-button activation disables every submit button in the form", () => {
  const handle = loadFeedback();
  const secondButton = new Element("button", { type: "submit" });
  handle.regular.appendChild(secondButton);
  handle.document.activeElement = new Element("input", { type: "text" });
  submit(handle.regular, handle.regularButton);

  for (const button of [handle.regularButton, secondButton]) {
    assert.equal(button.disabled, true);
    assert.ok(button._classes.has("btn-loading"));
  }
});

test("GET forms are ignored", () => {
  const handle = loadFeedback();
  handle.document.activeElement = handle.getButton;
  submit(handle.getForm, handle.getButton);

  assert.equal(handle.getButton.disabled, false);
  assert.equal(handle.getButton.children.length, 0);
});

test("flash toast auto-hides after the fade timeout chain", () => {
  const handle = loadFeedback({ withToast: true });
  const toast = handle.toast;
  assert.equal(toast.hidden, false);
  assert.equal(handle.timers.timeouts.length, 1, "one fade-out timer is scheduled");

  handle.timers.runAll();

  assert.ok(toast._classes.has("hide"), "toast fades out before hiding");
  assert.equal(toast.hidden, true, "toast is hidden after the timeout chain");
});

test("no toast means no timers are scheduled", () => {
  const handle = loadFeedback();
  assert.equal(handle.timers.timeouts.length, 0);
});
