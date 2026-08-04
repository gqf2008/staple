// Native HTML5 drag & drop for the board page. No external dependencies.
// Cards are draggable with data-issue-id; columns are drop targets with
// data-status. On drop, POST form-encoded to /issues/{id}/status/ui and
// reload. The existing move forms remain as a no-JS fallback.
(function () {
  "use strict";
  var dragging = null;

  document.querySelectorAll(".board-column").forEach(function (column) {
    column.addEventListener("dragover", function (event) {
      event.preventDefault();
      column.classList.add("drag-over");
    });
    column.addEventListener("dragleave", function () {
      column.classList.remove("drag-over");
    });
    column.addEventListener("drop", function (event) {
      event.preventDefault();
      column.classList.remove("drag-over");
      if (!dragging) return;
      var issueId = dragging.getAttribute("data-issue-id");
      var status = column.getAttribute("data-status");
      if (!issueId || !status) return;
      var body = new URLSearchParams();
      body.append("status", status);
      fetch("/issues/" + encodeURIComponent(issueId) + "/status/ui", {
        method: "POST",
        headers: { "Content-Type": "application/x-www-form-urlencoded" },
        body: body.toString(),
        credentials: "same-origin",
      }).then(function () {
        window.location.reload();
      }).catch(function () {
        window.location.reload();
      });
    });
  });

  document.querySelectorAll(".board-card").forEach(function (card) {
    card.setAttribute("draggable", "true");
    card.addEventListener("dragstart", function (event) {
      dragging = card;
      card.classList.add("dragging");
      if (event.dataTransfer) {
        event.dataTransfer.effectAllowed = "move";
      }
    });
    card.addEventListener("dragend", function () {
      card.classList.remove("dragging");
      dragging = null;
      document.querySelectorAll(".board-column").forEach(function (column) {
        column.classList.remove("drag-over");
      });
    });
  });
})();
