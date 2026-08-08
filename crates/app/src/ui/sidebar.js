// Sidebar behavior (upstream SidebarShell parity, approximated in SSR;
// issues #237 + #244 + #248).
//
// Desktop (>48rem): 240px sidebar with collapse to a 64px rail and
// drag/keyboard resize (208-420px), persisted.
//
// Narrow (<=48rem): off-canvas drawer approximation — the sidebar is fixed
// and translated off-screen (visibility hidden + inert while closed), a
// hamburger toggle opens it with a scrim, Esc/scrim click closes it, and the
// open state is persisted. One click handler dispatches by the current
// breakpoint and handlers stay bound across breakpoint changes.
(function () {
  function clampSidebarWidth(width) {
    var DEFAULT_WIDTH = 240;
    var MIN_WIDTH = 208;
    var MAX_WIDTH = 420;
    if (!Number.isFinite(width)) return DEFAULT_WIDTH;
    return Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, Math.round(width)));
  }

  function isNarrow() {
    return typeof window.matchMedia === "function" &&
      window.matchMedia("(max-width: 48rem)").matches;
  }

  document.addEventListener("DOMContentLoaded", function () {
    var sidebar = document.querySelector(".app-sidebar[data-collapsible]");
    var toggle = document.getElementById("sidebar-toggle");
    var resizer = document.getElementById("sidebar-resizer");
    var scrim = document.getElementById("sidebar-scrim");
    if (!sidebar || !toggle) return;
    var COLLAPSE_KEY = "staple.sidebar.collapsed";
    var WIDTH_KEY = "staple.sidebar.width";
    var MOBILE_KEY = "staple.sidebar.mobileOpen";

    function readStored(key, fallback) {
      try {
        var raw = localStorage.getItem(key);
        return raw === null || raw === "" ? fallback : raw;
      } catch (_) { return fallback; }
    }
    function writeStored(key, value) {
      try { localStorage.setItem(key, value); } catch (_) {}
    }

    var width = clampSidebarWidth(Number(readStored(WIDTH_KEY, 240)));
    var drawerOpen = false;
    var sidebarWidth = "240px";
    try {
      var cssWidth = getComputedStyle(sidebar).getPropertyValue("--sidebar-width").trim();
      if (cssWidth) sidebarWidth = cssWidth;
    } catch (_) {}

    function syncAria() {
      if (resizer) resizer.setAttribute("aria-valuenow", String(width));
    }

    function applyWidth() {
      var collapsed = sidebar.classList.contains("collapsed");
      // The toggle button width is owned by CSS (#sidebar-toggle = 32px);
      // only the sidebar width is managed inline here.
      if (collapsed) {
        sidebar.style.width = "";
      } else {
        sidebar.style.width = width + "px";
      }
      syncAria();
    }

    function apply(collapsed) {
      sidebar.classList.toggle("collapsed", collapsed);
      toggle.setAttribute("aria-expanded", collapsed ? "false" : "true");
      toggle.setAttribute("aria-label", collapsed ? toggle.dataset.expand || "Expand sidebar" : toggle.dataset.collapse || "Collapse sidebar");
      toggle.textContent = collapsed ? "»" : "«";
      applyWidth();
    }

    function setDrawer(open) {
      drawerOpen = open;
      sidebar.classList.toggle("drawer-open", open);
      if (scrim) scrim.hidden = !open;
      sidebar.inert = !open;
      toggle.setAttribute("aria-expanded", open ? "true" : "false");
      toggle.setAttribute("aria-label", open ? toggle.dataset.collapse || "Collapse sidebar" : toggle.dataset.expand || "Expand sidebar");
      toggle.textContent = open ? "×" : "☰";
      if (readStored(MOBILE_KEY, "0") !== (open ? "1" : "0")) {
        writeStored(MOBILE_KEY, open ? "1" : "0");
      }
    }

    function syncMode() {
      if (isNarrow()) {
        // Drawer mode: desktop collapse/width persistence is ignored.
        sidebar.classList.remove("collapsed");
        sidebar.style.width = sidebarWidth;
        // Narrow media query sizes the hamburger (40px); no inline width.
        setDrawer(readStored(MOBILE_KEY, "0") === "1");
      } else {
        drawerOpen = false;
        sidebar.inert = false;
        if (scrim) scrim.hidden = true;
        sidebar.classList.remove("drawer-open");
        apply(readStored(COLLAPSE_KEY, "0") === "1");
      }
    }

    // One toggle handler dispatches by the current breakpoint, so it keeps
    // working after a resize crosses the 48rem boundary.
    toggle.addEventListener("click", function () {
      if (isNarrow()) {
        setDrawer(!drawerOpen);
      } else {
        var collapsed = !sidebar.classList.contains("collapsed");
        apply(collapsed);
        writeStored(COLLAPSE_KEY, collapsed ? "1" : "0");
      }
    });

    if (scrim) {
      scrim.addEventListener("click", function () { setDrawer(false); });
    }
    document.addEventListener("keydown", function (event) {
      if (event.key === "Escape" && drawerOpen) setDrawer(false);
    });

    // Drag/keyboard resize: bound once, guarded by isNarrow() so it only
    // acts on desktop (drawer mode never drags).
    if (resizer) {
      var dragging = null;
      function endDrag(cancel) {
        if (!dragging) return;
        if (cancel) {
          width = dragging.startWidth;
          applyWidth();
        } else {
          writeStored(WIDTH_KEY, String(width));
        }
        sidebar.style.transition = "";
        dragging = null;
      }
      resizer.addEventListener("pointerdown", function (event) {
        if (sidebar.classList.contains("collapsed") || isNarrow()) return;
        dragging = { startX: event.clientX, startWidth: width };
        sidebar.style.transition = "none";
        if (resizer.setPointerCapture) resizer.setPointerCapture(event.pointerId);
        event.preventDefault();
      });
      window.addEventListener("pointermove", function (event) {
        if (!dragging) return;
        width = clampSidebarWidth(dragging.startWidth + (event.clientX - dragging.startX));
        sidebar.style.width = width + "px";
        syncAria();
      });
      window.addEventListener("pointerup", function () { endDrag(false); });
      window.addEventListener("pointercancel", function () { endDrag(true); });

      resizer.addEventListener("keydown", function (event) {
        if (sidebar.classList.contains("collapsed") || isNarrow()) return;
        var next = width;
        if (event.key === "ArrowRight") next = width + 16;
        else if (event.key === "ArrowLeft") next = width - 16;
        else if (event.key === "Home") next = 208;
        else if (event.key === "End") next = 420;
        else return;
        event.preventDefault();
        width = clampSidebarWidth(next);
        applyWidth();
        writeStored(WIDTH_KEY, String(width));
      });
    }

    syncMode();

    // Live re-sync when crossing the 48rem breakpoint.
    var mq = typeof window.matchMedia === "function" ? window.matchMedia("(max-width: 48rem)") : null;
    if (mq) {
      if (mq.addEventListener) mq.addEventListener("change", syncMode);
      else if (mq.addListener) mq.addListener(syncMode);
    }
  });

  // Exposed for the zero-dependency node:test suite.
  if (typeof module !== "undefined" && module.exports) {
    module.exports = { clampSidebarWidth: clampSidebarWidth };
  }
})();
