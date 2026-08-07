// Sidebar collapse + resizable width (upstream SidebarShell parity,
// approximated in SSR; issue #237 + #244). Collapses the 240px sidebar to a
// 64px rail, persists the choice, and supports drag-resizing the expanded
// width between 208px and 420px (localStorage "staple.sidebar.width"),
// including keyboard resizing (ArrowLeft/Right, Home/End) on the resizer.
// No-op when the markup is absent.
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
    if (!sidebar || !toggle) return;
    var COLLAPSE_KEY = "staple.sidebar.collapsed";
    var WIDTH_KEY = "staple.sidebar.width";

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

    function syncAria() {
      if (resizer) resizer.setAttribute("aria-valuenow", String(width));
    }

    function applyWidth() {
      if (sidebar.classList.contains("collapsed")) {
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

    apply(readStored(COLLAPSE_KEY, "0") === "1");
    toggle.addEventListener("click", function () {
      var collapsed = !sidebar.classList.contains("collapsed");
      apply(collapsed);
      writeStored(COLLAPSE_KEY, collapsed ? "1" : "0");
    });

    // Drag-resize (disabled while collapsed or on narrow screens, matching
    // the upstream mobile rail behavior). Width transitions are disabled
    // while dragging so the handle stays under the pointer (upstream note:
    // "Resizing the drag handle is likewise direct").
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

      // Keyboard resizing (upstream: ArrowLeft/Right step 16, Home/End).
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
  });

  // Exposed for the zero-dependency node:test suite.
  if (typeof module !== "undefined" && module.exports) {
    module.exports = { clampSidebarWidth: clampSidebarWidth };
  }
})();
