// Global command palette (Cmd/Ctrl+K). Filters the static page items rendered
// in the layout, appends matching issues from the current company, and
// navigates on Enter. No-op when the palette markup is absent.
document.addEventListener("DOMContentLoaded", function () {
  var palette = document.getElementById("command-palette");
  if (!palette) return;
  var input = document.getElementById("command-input");
  var list = document.getElementById("command-list");
  if (!input || !list) return;
  var panel = palette.querySelector(".command-palette-panel");
  var companyId = panel ? panel.getAttribute("data-company-id") || "" : "";
  var staticItems = Array.prototype.slice.call(list.querySelectorAll(".command-item"));
  var issueItems = [];
  var active = -1;
  var debounce = null;

  function visibleItems() {
    var result = [];
    staticItems.forEach(function (el) { if (!el.hidden) result.push(el); });
    issueItems.forEach(function (el) { if (!el.hidden) result.push(el); });
    return result;
  }

  function highlight() {
    var shown = visibleItems();
    shown.forEach(function (el, index) {
      el.classList.toggle("active", index === active);
    });
    if (active >= 0 && active < shown.length && shown[active].scrollIntoView) {
      shown[active].scrollIntoView({ block: "nearest" });
    }
  }

  function open() {
    palette.hidden = false;
    input.value = "";
    active = -1;
    staticItems.forEach(function (el) { el.hidden = false; });
    issueItems.forEach(function (el) { el.remove(); });
    issueItems = [];
    input.focus();
    highlight();
  }

  function close() {
    palette.hidden = true;
    active = -1;
  }

  function navigate() {
    var shown = visibleItems();
    if (active >= 0 && active < shown.length && shown[active].getAttribute("href")) {
      window.location.href = shown[active].getAttribute("href");
    }
  }

  function appendIssueItems(issues, query) {
    issueItems.forEach(function (el) { el.remove(); });
    issueItems = [];
    var lower = query.toLowerCase();
    issues.forEach(function (issue) {
      var identifier = issue.identifier || "";
      var title = issue.title || "";
      if (lower && title.toLowerCase().indexOf(lower) === -1 && identifier.toLowerCase().indexOf(lower) === -1) return;
      var el = document.createElement("a");
      el.className = "command-item";
      el.setAttribute("href", "/issues/" + encodeURIComponent(issue.id) + "?lang=" + encodeURIComponent(new URLSearchParams(window.location.search).get("lang") || "en"));
      el.setAttribute("data-kind", "issue");
      var idSpan = document.createElement("span");
      idSpan.className = "command-id";
      idSpan.textContent = identifier;
      el.appendChild(idSpan);
      el.appendChild(document.createTextNode(title));
      list.appendChild(el);
      issueItems.push(el);
    });
    highlight();
  }

  input.addEventListener("input", function () {
    var q = input.value.trim().toLowerCase();
    active = -1;
    staticItems.forEach(function (el) {
      el.hidden = el.textContent.toLowerCase().indexOf(q) === -1;
    });
    issueItems.forEach(function (el) { el.remove(); });
    issueItems = [];
    if (debounce) clearTimeout(debounce);
    if (companyId && q) {
      debounce = setTimeout(function () {
        fetch("/api/companies/" + encodeURIComponent(companyId) + "/issues")
          .then(function (r) { return r.json(); })
          .then(function (issues) { if (Array.isArray(issues)) appendIssueItems(issues, q); })
          .catch(function () {});
      }, 150);
    }
    highlight();
  });

  input.addEventListener("keydown", function (event) {
    var shown = visibleItems();
    if (event.key === "ArrowDown") { event.preventDefault(); active = Math.min(active + 1, shown.length - 1); highlight(); }
    else if (event.key === "ArrowUp") { event.preventDefault(); active = Math.max(active - 1, 0); highlight(); }
    else if (event.key === "Enter") { event.preventDefault(); navigate(); }
    else if (event.key === "Escape") { close(); }
  });

  document.addEventListener("keydown", function (event) {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      if (palette.hidden) open(); else close();
    }
  });

  palette.addEventListener("click", function (event) {
    if (event.target === palette) close();
  });
});
