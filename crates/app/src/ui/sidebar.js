// Sidebar collapse + resizable width (upstream SidebarShell parity,
// approximated in SSR; issue #237 + #244). Collapses the 240px sidebar to a
// 64px rail, persists the choice, and supports drag-resizing the expanded
// width between 208px and 420px (localStorage "staple.sidebar.width").
// No-op when the markup is absent.
(function () {
  function clampSidebarWidth(width) {
    var DEFAULT_WIDTH = 240;
    var MIN_WIDTH = 208;
    var MAX_WIDTH = 420;
    if (!Number.isFinite(width)) return DEFAULT_WIDTH;
    return Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, Math.round(width)));
  }

  document.addEventListener("DOMContentLoaded", function () {
    var sidebar = document.querySelector(".app-sidebar[data-collapsible]");
    var toggle = document.getElementById("sidebar-toggle");
    var resizer = document.getElementById("sidebar-resizer");
    if (!sidebar || !toggle) return;
    var COLLAPSE_KEY = "staple.sidebar.collapsed";
    var WIDTH_KEY = "staple.sidebar.width";

    function readStored(key, fallback) {
      try {
        var raw = localStorage.getItem(key);
        return raw === null ? fallback : raw;
      } catch (_) { return fallback; }
    }
    function writeStored(key, value) {
      try { localStorage.setItem(key, value); } catch (_) {}
    }

    var width = clampSidebarWidth(Number(readStored(WIDTH_KEY, 240)));

    function applyWidth() {
      if (sidebar.classList.contains("collapsed")) {
        sidebar.style.width = "";
      } else {
        sidebar.style.width = width + "px";
      }
    }

    function apply(collapsed) {
      sidebar.classList.toggle("collapsed", collapsed);
      toggle.setAttribute("aria-expanded", collapsed ? "false" : "true");
      toggle.setAttribute("aria-label", collapsed ? toggle.dataset.expand || "Expand sidebar" : toggle.dataset.collapse || "Collapse sidebar");
      toggle.textContent = collapsed ? "»" : "«";
      applyWidth();
    }

    apply(readStored(COLLAPSE_KEY, "0") === "1");
    toggle.addEventListener("click", function () {
      var collapsed = !sidebar.classList.contains("collapsed");
      apply(collapsed);
      writeStored(COLLAPSE_KEY, collapsed ? "1" : "0");
    });

    // Drag-resize (disabled while collapsed, matching upstream).
    if (resizer) {
      var dragging = null;
      resizer.addEventListener("pointerdown", function (event) {
        if (sidebar.classList.contains("collapsed")) return;
        dragging = { startX: event.clientX, startWidth: width };
        if (resizer.setPointerCapture) resizer.setPointerCapture(event.pointerId);
        event.preventDefault();
      });
      window.addEventListener("pointermove", function (event) {
        if (!dragging) return;
        width = clampSidebarWidth(dragging.startWidth + (event.clientX - dragging.startX));
        sidebar.style.width = width + "px";
      });
      window.addEventListener("pointerup", function () {
        if (!dragging) return;
        writeStored(WIDTH_KEY, String(width));
        dragging = null;
      });
      window.addEventListener("pointercancel", function () {
        dragging = null;
        applyWidth();
      });
    }
  });

  // Exposed for the zero-dependency node:test suite.
  if (typeof module !== "undefined" && module.exports) {
    module.exports = { clampSidebarWidth: clampSidebarWidth };
  }
})();
