// Theme switching (upstream dark-mode parity, issue #242): cycles
// system -> light -> dark, persists the choice in localStorage, and keeps
// following the OS scheme while in "system" mode. The pre-paint inline
// script in layout.rs applies the initial class so there is no light flash.
document.addEventListener("DOMContentLoaded", function () {
  var root = document.documentElement;
  var toggle = document.getElementById("theme-toggle");
  if (!root || !toggle) return;
  var STORAGE_KEY = "staple.theme";
  var ORDER = ["system", "light", "dark"];
  function systemDark() {
    return typeof window.matchMedia === "function" && window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  function apply(mode) {
    var dark = mode === "dark" || (mode === "system" && systemDark());
    root.classList.toggle("dark", dark);
    var labelKey = mode === "system" ? toggle.dataset.themeSystem
      : mode === "light" ? toggle.dataset.themeLight : toggle.dataset.themeDark;
    toggle.setAttribute("aria-label", labelKey || mode);
    toggle.textContent = mode === "system" ? "\u25d0" : mode === "light" ? "\u2600" : "\u263e";
    toggle.dataset.mode = mode;
  }
  function persist(mode) {
    try { localStorage.setItem(STORAGE_KEY, mode); } catch (_) {}
  }
  var mode = "system";
  try {
    var stored = localStorage.getItem(STORAGE_KEY);
    if (ORDER.indexOf(stored) !== -1) mode = stored;
  } catch (_) {}
  apply(mode);
  toggle.addEventListener("click", function () {
    var next = ORDER[(ORDER.indexOf(toggle.dataset.mode || "system") + 1) % ORDER.length];
    apply(next);
    persist(next);
  });
  if (typeof window.matchMedia === "function") {
    var mq = window.matchMedia("(prefers-color-scheme: dark)");
    var onChange = function () {
      if ((toggle.dataset.mode || "system") === "system") {
        root.classList.toggle("dark", systemDark());
      }
    };
    if (mq.addEventListener) mq.addEventListener("change", onChange);
    else if (mq.addListener) mq.addListener(onChange);
  }
});
