// UI/UX alignment end-to-end tests (issue #254).
//
// Runs against a running Staple server (default http://127.0.0.1:3109) with
// Playwright, asserts the computed-style invariants aligned to the upstream
// runtime specs (doc/plans/2026-08-07-upstream-runtime-shots.md §3), saves
// screenshots and a JSON report.
//
// Env:
//   BASE_URL            server base url (default http://127.0.0.1:3109)
//   PW_EXECUTABLE       optional chromium executable path
//   E2E_OUT_DIR         screenshot dir (default target/ui-e2e-screenshots)
//   E2E_REPORT          report path (default target/ui-e2e-report.json)
//
// Run: make ui-e2e   (or: BASE_URL=... node --test scripts/ui_e2e/alignment.test.mjs)

import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const require = createRequire(import.meta.url);
const { chromium } = require("playwright");

const BASE_URL = process.env.BASE_URL || "http://127.0.0.1:3109";
const EXECUTABLE = process.env.PW_EXECUTABLE || undefined;
const OUT_DIR = process.env.E2E_OUT_DIR || "target/ui-e2e-screenshots";
const REPORT_PATH = process.env.E2E_REPORT || "target/ui-e2e-report.json";

const CID = process.env.E2E_COMPANY_ID || null; // discovered if not set
const state = { companyId: CID, issueId: null };
const results = [];
let browser = null;
let ctx = null;

function record(name, ok, detail) {
  results.push({ name, ok, ...(detail ? { detail } : {}) });
}

async function page(viewport = { width: 1440, height: 900 }) {
  return browser.newPage({ viewport });
}

async function firstCompanyId() {
  const r = await fetch(`${BASE_URL}/api/companies`);
  const body = await r.json();
  if (Array.isArray(body) && body.length > 0) return body[0].id;
  // self-seed an E2E company through the API
  const created = await fetch(`${BASE_URL}/api/companies`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name: "UI E2E", description: "created by ui_e2e suite", issuePrefix: "E2E" }),
  });
  const createdBody = await created.json();
  if (!created.ok || !createdBody.id) throw new Error(`seed company failed: ${created.status} ${JSON.stringify(createdBody)}`);
  return createdBody.id;
}

async function ensureApproval(companyId) {
  const r = await fetch(`${BASE_URL}/api/companies/${companyId}/approvals`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ type: "budget_override_required", payload: {} }),
  });
  const body = await r.json();
  if (!r.ok || !body.id) throw new Error(`seed approval failed: ${r.status} ${JSON.stringify(body)}`);
  return body.id;
}

async function ensureIssue(companyId) {
  const r = await fetch(`${BASE_URL}/api/companies/${companyId}/issues`);
  const body = await r.json();
  if (Array.isArray(body) && body.length > 0) return body[0].id;
  const created = await fetch(`${BASE_URL}/api/companies/${companyId}/issues`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title: "UI E2E issue", description: "created by ui_e2e suite" }),
  });
  const createdBody = await created.json();
  if (!created.ok || !createdBody.id) throw new Error(`seed issue failed: ${created.status} ${JSON.stringify(createdBody)}`);
  return createdBody.id;
}

test.before(async () => {
  mkdirSync(OUT_DIR, { recursive: true });
  browser = await (EXECUTABLE ? chromium.launch({ executablePath: EXECUTABLE }) : chromium.launch());
  state.companyId = CID || (await firstCompanyId());
  state.issueId = await ensureIssue(state.companyId);
  await ensureApproval(state.companyId); // drives the sidebar inbox badge
});

test.after(async () => {
  if (browser) await browser.close();
  writeFileSync(REPORT_PATH, JSON.stringify({ generatedAt: new Date().toISOString(), baseUrl: BASE_URL, results }, null, 2));
  const failed = results.filter((r) => !r.ok).length;
  console.log(`\nUI E2E: ${results.length - failed}/${results.length} passed -> ${REPORT_PATH}`);
});

test("global tokens & font (light)", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.goto(`${BASE_URL}/companies/${companyId}/board`, { waitUntil: "networkidle" });
  await p.waitForTimeout(400);
  const m = await p.evaluate(() => {
    const root = getComputedStyle(document.documentElement);
    const body = getComputedStyle(document.body);
    return {
      font: body.fontFamily,
      bodyBg: body.backgroundColor,
      bodyFg: body.color,
      primary: root.getPropertyValue("--color-primary").trim(),
      sidebarW: document.querySelector(".app-sidebar").getBoundingClientRect().width,
    };
  });
  await p.screenshot({ path: join(OUT_DIR, "01-board-light.png") });
  await p.close();
  try {
    assert.ok(m.font.startsWith("InterVariable"), `font=${m.font}`);
    assert.equal(m.bodyBg, "oklch(1 0 0)");
    assert.equal(m.bodyFg, "oklch(0.145 0 0)");
    assert.equal(m.primary, "oklch(0.205 0 0)");
    assert.equal(Math.round(m.sidebarW), 240);
    record("global tokens & font (light)", true, m);
  } catch (e) { record("global tokens & font (light)", false, { error: e.message, ...m }); throw e; }
});

test("primary button specs", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.goto(`${BASE_URL}/companies/${companyId}/board/chat`, { waitUntil: "networkidle" });
  await p.waitForTimeout(300);
  const m = await p.evaluate(() => {
    const btn = document.querySelector('button[type="submit"]');
    if (!btn) return null;
    const s = getComputedStyle(btn);
    return { h: btn.getBoundingClientRect().height, radius: s.borderRadius, fw: s.fontWeight, bg: s.backgroundColor, color: s.color };
  });
  await p.close();
  try {
    assert.ok(m, "no primary button found");
    assert.equal(Math.round(m.h), 40);
    assert.equal(m.radius, "6.4px");
    assert.equal(m.fw, "500");
    assert.equal(m.bg, "oklch(0.205 0 0)");
    assert.equal(m.color, "oklch(0.985 0 0)");
    record("primary button specs", true, m);
  } catch (e) { record("primary button specs", false, { error: e.message, ...m }); throw e; }
});

test("badge specs", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.goto(`${BASE_URL}/companies/${companyId}/issues`, { waitUntil: "networkidle" });
  await p.waitForTimeout(300);
  const m = await p.evaluate(() => {
    const el = document.querySelector(".badge");
    if (!el) return null;
    const s = getComputedStyle(el);
    return { h: el.getBoundingClientRect().height, radius: s.borderRadius, ws: s.whiteSpace, fw: s.fontWeight };
  });
  await p.screenshot({ path: join(OUT_DIR, "02-inbox.png") });
  await p.close();
  try {
    assert.ok(m, "no badge found");
    assert.equal(Math.round(m.h), 22);
    assert.ok(parseFloat(m.radius) > 1000, `radius=${m.radius}`);
    assert.equal(m.ws, "nowrap");
    assert.equal(m.fw, "500");
    record("badge specs", true, m);
  } catch (e) { record("badge specs", false, { error: e.message, ...m }); throw e; }
});

test("command palette specs (open/close)", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.goto(`${BASE_URL}/companies/${companyId}/board`, { waitUntil: "networkidle" });
  await p.waitForTimeout(300);
  await p.keyboard.press("Meta+k");
  await p.waitForTimeout(250);
  const m = await p.evaluate(() => {
    const panel = document.querySelector(".command-palette-panel");
    if (!panel) return null;
    const input = document.querySelector(".command-palette-input");
    const item = document.querySelector(".command-item");
    const ps = getComputedStyle(panel);
    const is = getComputedStyle(input);
    return {
      w: panel.getBoundingClientRect().width,
      radius: ps.borderRadius,
      shadow: ps.boxShadow,
      inputH: input.getBoundingClientRect().height,
      inputPad: `${is.paddingTop} ${is.paddingLeft}`,
      inputRadius: is.borderRadius,
      itemPad: item ? getComputedStyle(item).padding : null,
    };
  });
  await p.screenshot({ path: join(OUT_DIR, "03-command-palette.png") });
  await p.keyboard.press("Escape");
  const closed = await p.evaluate(() => document.querySelector(".command-palette").hidden);
  await p.close();
  try {
    assert.ok(m, "palette panel not rendered");
    assert.equal(Math.round(m.w), 512);
    assert.equal(m.radius, "8px");
    assert.ok(m.shadow.includes("0px 10px 15px"), `shadow=${m.shadow}`);
    assert.equal(Math.round(m.inputH), 48);
    assert.equal(m.inputPad, "0px 12px");
    assert.equal(m.inputRadius, "0px");
    assert.equal(m.itemPad, "12px 8px");
    assert.equal(closed, true, "Escape should close");
    record("command palette specs (open/close)", true, m);
  } catch (e) { record("command palette specs (open/close)", false, { error: e.message, ...m }); throw e; }
});

test("board card specs", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.goto(`${BASE_URL}/companies/${companyId}/board`, { waitUntil: "networkidle" });
  await p.waitForTimeout(300);
  const m = await p.evaluate(() => {
    const el = document.querySelector(".board-card");
    if (!el) return null;
    const s = getComputedStyle(el);
    return { pad: s.padding, radius: s.borderRadius };
  });
  await p.close();
  try {
    assert.ok(m, "no board card");
    assert.equal(m.pad, "10px");
    assert.equal(m.radius, "8px");
    record("board card specs", true, m);
  } catch (e) { record("board card specs", false, { error: e.message, ...m }); throw e; }
});

test("issue detail sections", async () => {
  const companyId = state.companyId;
  const issueId = state.issueId;
  const p = await page();
  await p.goto(`${BASE_URL}/issues/${issueId}`, { waitUntil: "networkidle" });
  await p.waitForTimeout(300);
  const m = await p.evaluate(() => {
    const el = document.querySelector(".issue-section");
    if (!el) return null;
    const s = getComputedStyle(el);
    return { pad: s.padding, radius: s.borderRadius };
  });
  await p.screenshot({ path: join(OUT_DIR, "04-issue-detail.png") });
  await p.close();
  try {
    assert.ok(m, "no issue-section");
    assert.equal(m.radius, "8px");
    assert.equal(m.pad, "24px");
    record("issue detail sections", true, m);
  } catch (e) { record("issue detail sections", false, { error: e.message, ...m }); throw e; }
});

test("dark theme tokens", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.addInitScript(() => { try { localStorage.setItem("staple.theme", "dark"); } catch (_) {} });
  await p.goto(`${BASE_URL}/companies/${companyId}/board`, { waitUntil: "networkidle" });
  await p.waitForTimeout(500);
  const m = await p.evaluate(() => {
    const root = getComputedStyle(document.documentElement);
    const body = getComputedStyle(document.body);
    return {
      dark: document.documentElement.classList.contains("dark"),
      bg: body.backgroundColor,
      fg: body.color,
      primary: root.getPropertyValue("--color-primary").trim(),
    };
  });
  await p.screenshot({ path: join(OUT_DIR, "05-board-dark.png") });
  await p.close();
  try {
    assert.equal(m.dark, true);
    assert.equal(m.bg, "oklch(0.145 0 0)");
    assert.equal(m.fg, "oklch(0.985 0 0)");
    assert.equal(m.primary, "oklch(0.922 0 0)");
    record("dark theme tokens", true, m);
  } catch (e) { record("dark theme tokens", false, { error: e.message, ...m }); throw e; }
});

test("narrow drawer + no horizontal overflow", async () => {
  const companyId = state.companyId;
  const p = await page({ width: 375, height: 800 });
  await p.goto(`${BASE_URL}/companies/${companyId}/board`, { waitUntil: "networkidle" });
  await p.waitForTimeout(400);
  const closed = await p.evaluate(() => {
    const sb = document.querySelector(".app-sidebar");
    return {
      drawerOpen: sb.classList.contains("drawer-open"),
      transform: getComputedStyle(sb).transform,
      overflowX: document.documentElement.scrollWidth > window.innerWidth,
      toggleText: document.getElementById("sidebar-toggle").textContent,
    };
  });
  await p.click("#sidebar-toggle");
  await p.waitForTimeout(400);
  const opened = await p.evaluate(() => ({
    drawerOpen: document.querySelector(".app-sidebar").classList.contains("drawer-open"),
    scrimHidden: document.getElementById("sidebar-scrim").hidden,
  }));
  await p.screenshot({ path: join(OUT_DIR, "06-narrow-drawer-open.png") });
  await p.close();
  try {
    assert.equal(closed.drawerOpen, false);
    assert.ok(closed.transform.includes("-240") || closed.transform.includes("-100"), `transform=${closed.transform}`);
    assert.equal(closed.overflowX, false);
    assert.equal(closed.toggleText, "☰");
    assert.equal(opened.drawerOpen, true);
    assert.equal(opened.scrimHidden, false);
    record("narrow drawer + no horizontal overflow", true, { closed, opened });
  } catch (e) { record("narrow drawer + no horizontal overflow", false, { error: e.message, closed, opened }); throw e; }
});
