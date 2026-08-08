// Issues toolbar (issue #268): "+ New Task" toggles the inline quick-create
// form, and the search input filters the issue list client-side by
// identifier/title. Vanilla JS in the same style as board.js / ui_feedback.js;
// no dependencies.
document.addEventListener("DOMContentLoaded", function () {
  var toggle = document.getElementById("new-issue-toggle");
  var form = document.getElementById("new-issue-form");
  var search = document.getElementById("issue-search");
  var list = document.getElementById("issue-list");

  if (toggle && form) {
    toggle.addEventListener("click", function () {
      var hidden = form.hidden;
      form.hidden = !hidden;
      if (form.hidden) {
        toggle.setAttribute("aria-expanded", "false");
      } else {
        toggle.setAttribute("aria-expanded", "true");
        var input = document.getElementById("new-issue-title");
        if (input) input.focus();
      }
    });
  }

  if (search && list) {
    search.addEventListener("input", function () {
      var q = search.value.trim().toLowerCase();
      var rows = list.querySelectorAll("li:not(#issue-empty)");
      var visible = 0;
      for (var i = 0; i < rows.length; i++) {
        var row = rows[i];
        var hay = (row.getAttribute("data-search") || row.textContent || "").toLowerCase();
        var show = q === "" || hay.indexOf(q) !== -1;
        row.hidden = !show;
        if (show) visible += 1;
      }
      var empty = document.getElementById("issue-empty");
      if (empty) empty.hidden = visible !== 0;
    });
  }
});
