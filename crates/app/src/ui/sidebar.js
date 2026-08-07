// Sidebar collapse (upstream SidebarShell rail parity, approximated in SSR):
// toggles the 240px sidebar down to a 64px rail, persists the choice, and
// updates the toggle affordance. No-op when the markup is absent.
document.addEventListener("DOMContentLoaded", function () {
  var sidebar = document.querySelector(".app-sidebar[data-collapsible]");
  var toggle = document.getElementById("sidebar-toggle");
  if (!sidebar || !toggle) return;
  var STORAGE_KEY = "staple.sidebar.collapsed";
  function apply(collapsed) {
    sidebar.classList.toggle("collapsed", collapsed);
    toggle.setAttribute("aria-expanded", collapsed ? "false" : "true");
    toggle.setAttribute("aria-label", collapsed ? toggle.dataset.expand || "Expand sidebar" : toggle.dataset.collapse || "Collapse sidebar");
    toggle.textContent = collapsed ? "»" : "«";
  }
  try {
    if (localStorage.getItem(STORAGE_KEY) === "1") apply(true);
  } catch (_) { /* storage unavailable: keep expanded */ }
  toggle.addEventListener("click", function () {
    var collapsed = !sidebar.classList.contains("collapsed");
    apply(collapsed);
    try { localStorage.setItem(STORAGE_KEY, collapsed ? "1" : "0"); } catch (_) {}
  });
});
