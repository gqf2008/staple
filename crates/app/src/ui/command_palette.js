// Global command palette (Cmd/Ctrl+K). Filters the static page items rendered
// in the layout, appends matching issues from the current company (capped),
// and navigates on Enter. No-op when the palette markup is absent.
document.addEventListener("DOMContentLoaded", function () {
  var palette = document.getElementById("command-palette");
  if (!palette) return;
  var input = document.getElementById("command-input");
  var list = document.getElementById("command-list");
  var empty = document.getElementById("command-empty");
  if (!input || !list) return;
  var panel = palette.querySelector(".command-palette-panel");
  var companyId = panel ? panel.getAttribute("data-company-id") || "" : "";
  var staticItems = Array.prototype.slice.call(list.querySelectorAll(".command-item"));
  var issueItems = [];
  var active = -1;
  var debounce = null;
  var requestToken = 0;
  var ISSUE_LIMIT = 50;

  function visibleItems() {
    var result = [];
    staticItems.forEach(function (el) { if (!el.hidden) result.push(el); });
    issueItems.forEach(function (el) { if (!el.hidden) result.push(el); });
    return result;
  }

  function syncEmpty() {
    if (empty) empty.hidden = visibleItems().length !== 0;
  }

  function highlight() {
    var shown = visibleItems();
    var prev = list.querySelector("#command-active-item");
    if (prev) prev.removeAttribute("id");
    shown.forEach(function (el, index) {
      el.classList.toggle("active", index === active);
      if (index === active) el.setAttribute("id", "command-active-item");
    });
    if (input) {
      if (active >= 0 && active < shown.length) {
        input.setAttribute("aria-activedescendant", "command-active-item");
      } else {
        input.removeAttribute("aria-activedescendant");
      }
    }
    if (active >= 0 && active < shown.length && shown[active].scrollIntoView) {
      shown[active].scrollIntoView({ block: "nearest" });
    }
    syncEmpty();
  }

  function open() {
    if (debounce) clearTimeout(debounce);
    requestToken += 1;
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
    if (debounce) clearTimeout(debounce);
    requestToken += 1;
    palette.hidden = true;
    active = -1;
  }

  function navigate() {
    var shown = visibleItems();
    if (active >= 0 && active < shown.length && shown[active].getAttribute("href")) {
      window.location.href = shown[active].getAttribute("href");
    }
  }

  function appendIssueItems(issues, query, token) {
    if (token !== requestToken || palette.hidden) return;
    issueItems.forEach(function (el) { el.remove(); });
    issueItems = [];
    var lower = query.toLowerCase();
    var matched = 0;
    issues.forEach(function (issue) {
      if (matched >= ISSUE_LIMIT) return;
      var identifier = issue.identifier || "";
      var title = issue.title || "";
      if (lower && title.toLowerCase().indexOf(lower) === -1 && identifier.toLowerCase().indexOf(lower) === -1) return;
      var el = document.createElement("a");
      el.className = "command-item";
      var lang = new URLSearchParams(window.location.search).get("lang") || "en";
      el.setAttribute("href", "/issues/" + encodeURIComponent(issue.id) + "?lang=" + encodeURIComponent(lang));
      var idSpan = document.createElement("span");
      idSpan.className = "command-id";
      idSpan.textContent = identifier;
      el.appendChild(idSpan);
      el.appendChild(document.createTextNode(title));
      list.appendChild(el);
      issueItems.push(el);
      matched += 1;
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
      var token = ++requestToken;
      debounce = setTimeout(function () {
        fetch("/api/companies/" + encodeURIComponent(companyId) + "/issues")
          .then(function (r) { return r.json(); })
          .then(function (issues) { if (Array.isArray(issues)) appendIssueItems(issues, q, token); })
          .catch(function () {});
      }, 150);
    } else {
      requestToken += 1;
    }
    highlight();
  });

  input.addEventListener("keydown", function (event) {
    var shown = visibleItems();
    if (event.key === "ArrowDown") { event.preventDefault(); active = Math.min(active + 1, shown.length - 1); highlight(); }
    else if (event.key === "ArrowUp") {
      event.preventDefault();
      active = active <= 0 ? (shown.length ? shown.length - 1 : -1) : active - 1;
      highlight();
    }
    else if (event.key === "Enter") { event.preventDefault(); navigate(); }
    else if (event.key === "Escape") { close(); }
    else if (event.key === "Tab") {
      // Keep focus inside the palette.
      event.preventDefault();
      input.focus();
    }
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
