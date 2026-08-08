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

async function waitForEval(pg, fn, timeoutMs = 4000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (await pg.evaluate(fn)) return true;
    await pg.waitForTimeout(50);
  }
  return false;
}

async function firstCompanyId() {
  // The server may still be applying schema migrations right after boot;
  // retry until the companies API answers without 500.
  let body = null;
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const r = await fetch(`${BASE_URL}/api/companies`);
    if (r.status === 200) { body = await r.json(); break; }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  if (!body) throw new Error(`companies API unavailable: ${BASE_URL}`);
  if (Array.isArray(body) && body.length > 0) return body[0].id;
  // self-seed an E2E company through the API
  const created = await fetch(`${BASE_URL}/api/companies`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name: "UI E2E", description: "created by ui_e2e suite" }),
  });
  const createdBody = await created.json();
  if (!created.ok || !createdBody.id) throw new Error(`seed company failed: ${created.status} ${JSON.stringify(createdBody)}`);
  return createdBody.id;
}

async function ensureApproval(companyId) {
  const existing = await fetch(`${BASE_URL}/api/companies/${companyId}/approvals`).then((r) => r.json());
  if (Array.isArray(existing)) {
    const pending = existing.find((a) => a.status === "pending");
    if (pending) return pending.id;
  }
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
  const m = await p.evaluate(async () => {
    await document.fonts.ready;
    const root = getComputedStyle(document.documentElement);
    const body = getComputedStyle(document.body);
    return {
      font: body.fontFamily,
      fontLoaded: document.fonts.check('16px "InterVariable"'),
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
    assert.equal(m.fontLoaded, true, "InterVariable font file must load");
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
  assert.ok(
    await waitForEval(p, () => {
      const palette = document.querySelector(".command-palette");
      return palette && !palette.hidden;
    }),
    "palette should open",
  );
  await p.waitForTimeout(200);
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
  const closed = await waitForEval(p, () => document.querySelector(".command-palette").hidden);
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

test("stack-form buttons keep content width (chat send)", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.goto(`${BASE_URL}/companies/${companyId}/board/chat`, { waitUntil: "networkidle" });
  await p.waitForTimeout(400);
  const m = await p.evaluate(() => {
    const btn = document.querySelector('button[type="submit"]');
    const form = btn ? btn.closest("form") : null;
    const textarea = document.querySelector('textarea[name="message"]');
    return {
      btnW: btn ? Math.round(btn.getBoundingClientRect().width) : null,
      formW: form ? Math.round(form.getBoundingClientRect().width) : null,
      textareaW: textarea ? Math.round(textarea.getBoundingClientRect().width) : null,
    };
  });
  await p.close();
  try {
    assert.ok(m && m.btnW != null, "chat submit button not found");
    assert.ok(m.btnW < 120, `chat send button should be content width, got ${m.btnW}px`);
    assert.ok(m.textareaW != null && m.formW != null && m.textareaW >= m.formW - 2, `textarea should stay full width (${m.textareaW}/${m.formW})`);
    record("stack-form buttons keep content width (chat send)", true, m);
  } catch (e) { record("stack-form buttons keep content width (chat send)", false, { error: e.message, ...m }); throw e; }
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

test("sidebar icons render visibly (stroke outline, rail column)", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.goto(`${BASE_URL}/companies/${companyId}/board`, { waitUntil: "networkidle" });
  await p.waitForTimeout(400);
  const m = await p.evaluate(() => {
    const sb = document.querySelector(".app-sidebar");
    const svgs = Array.from(sb.querySelectorAll("a svg"));
    const first = svgs[0];
    const s = getComputedStyle(first);
    const rect = first.getBoundingClientRect();
    const iconPixels = (() => {
      const svg = first;
      const rect = svg.getBoundingClientRect();
      const paths = svg.querySelectorAll("path, line, rect, circle, polyline, polygon, ellipse");
      return paths.length;
    })();
    const toggle = document.getElementById("sidebar-toggle");
    toggle.click();
    // synchronous class toggling is applied by the click handler
    const collapsed = sb.classList.contains("collapsed");
    const labelHidden = collapsed ? getComputedStyle(sb.querySelector(".nav-label")).visibility : null;
    const iconVisible = collapsed ? getComputedStyle(svgs[0]).visibility : null;
    return {
      svgCount: svgs.length,
      iconW: Math.round(rect.width), iconH: Math.round(rect.height),
      stroke: s.stroke, fill: s.fill,
      paths: iconPixels,
      collapsed, labelHidden, iconVisible,
      overflowX: document.documentElement.scrollWidth > window.innerWidth,
    };
  });
  try {
    assert.equal(m.svgCount, 24, `expected 24 sidebar icons (23 nav + brand), got ${m.svgCount}`);
    assert.equal(m.iconW, 16);
    assert.equal(m.iconH, 16);
    assert.notEqual(m.stroke, "none", "icons must be stroked (Feather outline)");
    assert.equal(m.fill, "none");
    assert.ok(m.paths > 0, "icon should contain shape elements");
    assert.equal(m.collapsed, true);
    assert.equal(m.labelHidden, "hidden");
    assert.equal(m.iconVisible, "visible");
    assert.equal(m.overflowX, false);
    // Wait out the 160ms width transition before measuring the rail position.
    await p.waitForTimeout(300);
    const center = await p.evaluate(() => {
      const r = document.querySelector(".app-sidebar a svg").getBoundingClientRect();
      return r.left + r.width / 2;
    });
    assert.ok(Math.abs(center - 32) <= 6, `rail icon should be centered (~32px), got ${center}`);
    record("sidebar icons render visibly (stroke outline, rail column)", true, { ...m, iconCenterX: center });
    await p.close();
  } catch (e) { record("sidebar icons render visibly (stroke outline, rail column)", false, { error: e.message, ...m }); throw e; }
});

test("global link baseline + sidebar active highlight", async () => {
  const companyId = state.companyId;
  const p = await page();
  await p.goto(`${BASE_URL}/companies/${companyId}/board`, { waitUntil: "networkidle" });
  await waitForEval(p, () => document.querySelectorAll(".app-sidebar a.active").length === 1);
  const m = await p.evaluate(() => {
    const sb = document.querySelector(".app-sidebar");
    const activeLinks = Array.from(sb.querySelectorAll("a.active"));
    const active = activeLinks[0] || null;
    const accent = getComputedStyle(document.documentElement).getPropertyValue("--color-accent").trim();
    const foreground = getComputedStyle(document.documentElement).getPropertyValue("--color-foreground").trim();
    const as = active ? getComputedStyle(active) : null;
    const defaultBlue = Array.from(document.querySelectorAll("a")).filter((a) => {
      if (a.closest(".command-palette")) return false; // palette is hidden
      const c = getComputedStyle(a).color;
      const t = getComputedStyle(a).textDecorationLine || "";
      return c === "rgb(0, 0, 238)" || c === "rgb(0,0,238)" || t.includes("underline");
    }).length;
    return {
      activeCount: activeLinks.length,
      activeHref: active ? active.getAttribute("href") : null,
      activeBg: as ? as.backgroundColor : null,
      activeColor: as ? as.color : null,
      activeRadius: as ? as.borderRadius : null,
      accent, foreground, defaultBlue,
      globalBase: Array.from(document.querySelectorAll("style")).some(
        (st) => st.textContent.includes("a { color: var(--color-foreground); text-decoration: none; }")
      ),
    };
  });
  try {
    assert.equal(m.globalBase, true, "global link baseline CSS missing");
    assert.equal(m.activeCount, 1, `expected exactly one active sidebar link, got ${m.activeCount}`);
    assert.ok(m.activeHref && m.activeHref.includes(`/companies/${companyId}/board`), `active href=${m.activeHref}`);
    assert.equal(m.activeBg, m.accent, `active bg ${m.activeBg} != accent ${m.accent}`);
    assert.equal(m.activeColor, m.foreground, `active color ${m.activeColor} != foreground ${m.foreground}`);
    assert.equal(m.activeRadius, "8px", `active radius ${m.activeRadius} != 8px (upstream rounded-lg)`);
    assert.equal(m.defaultBlue, 0, `default browser-blue/underline links remain: ${m.defaultBlue}`);
    record("global link baseline + sidebar active highlight", true, m);
    await p.close();
  } catch (e) { record("global link baseline + sidebar active highlight", false, { error: e.message, ...m }); throw e; }
});

test("root page marks brand + companies active (no stray highlight)", async () => {
  const p = await page();
  await p.goto(`${BASE_URL}/`, { waitUntil: "networkidle" });
  await waitForEval(p, () => document.querySelectorAll(".app-sidebar a.active").length === 2);
  const m = await p.evaluate(() => ({
    activeCount: document.querySelectorAll(".app-sidebar a.active").length,
    activeTexts: Array.from(document.querySelectorAll(".app-sidebar a.active")).map((a) => (a.textContent || "").trim().slice(0, 24)),
    brandActive: (document.querySelector(".app-sidebar a.brand") || {}).classList?.contains("active") === true,
  }));
  try {
    assert.equal(m.activeCount, 2, `root expected brand + companies active, got ${m.activeCount}`);
    assert.equal(m.brandActive, true, "brand must be active on root");
    assert.ok(m.activeTexts.some((t) => t.includes("Companies")), `companies link not active: ${m.activeTexts}`);
    record("root page marks brand + companies active (no stray highlight)", true, m);
    await p.close();
  } catch (e) { record("root page marks brand + companies active (no stray highlight)", false, { error: e.message, ...m }); throw e; }
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
  const opened = await waitForEval(p, () => {
    const sb = document.querySelector(".app-sidebar");
    const t = getComputedStyle(sb).transform;
    return sb.classList.contains("drawer-open") && (t === "none" || t === "matrix(1, 0, 0, 1, 0, 0)");
  }, 4000)
    ? await p.evaluate(() => ({
        drawerOpen: document.querySelector(".app-sidebar").classList.contains("drawer-open"),
        scrimHidden: document.getElementById("sidebar-scrim").hidden,
      }))
    : { drawerOpen: false, scrimHidden: true };
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
