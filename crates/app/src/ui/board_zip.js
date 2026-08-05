// Company zip portability: preview then apply an uploaded archive. No-op
// when the zip form is not present.
document.addEventListener("DOMContentLoaded", function () {
  var form = document.getElementById("zip-form");
  var preview = document.getElementById("zip-preview");
  var fileInput = document.getElementById("zip-file");
  if (!form || !preview || !fileInput) return;

  var companyInput = document.querySelector('input[name="company_id"]');
  var companyId = companyInput ? companyInput.value : "";

  form.addEventListener("submit", async function (event) {
    event.preventDefault();
    var file = fileInput.files && fileInput.files[0];
    if (!file) return;
    var strategy = (form.querySelector('select[name="strategy"]') || {}).value || "skip";
    var body = await file.arrayBuffer();

    var response = await fetch("/api/companies/" + companyId + "/import/archive/preview", {
      method: "POST",
      headers: { "Content-Type": "application/octet-stream" },
      body: body
    });
    if (!response.ok) {
      preview.textContent = "Preview error " + response.status;
      return;
    }
    var data = await response.json();
    var tables = (data.manifest && data.manifest.tables || []).map(function (t) {
      return t.name + " (" + t.rows + ")";
    });
    var existing = data.existing || {};
    var conflictLines = Object.keys(existing)
      .filter(function (key) { return (existing[key] || 0) > 0; })
      .map(function (key) { return key + ": " + existing[key]; });
    var summary = document.createElement("p");
    summary.textContent = "Tables: " + tables.join(", ") +
      (conflictLines.length ? " | Existing rows: " + conflictLines.join(", ") : " | target empty");

    var tree = document.createElement("ul");
    function renderTree(nodes, parent) {
      (nodes || []).forEach(function (node) {
        var li = document.createElement("li");
        if (node.type === "dir") {
          li.textContent = "📁 " + node.name + "/";
          var childList = document.createElement("ul");
          renderTree(node.children || [], childList);
          li.appendChild(childList);
        } else {
          li.textContent = "📄 " + node.name + " (" + (node.size || 0) + " bytes)";
        }
        parent.appendChild(li);
      });
    }
    renderTree(data.filesTree || [], tree);

    preview.innerHTML = "";
    preview.appendChild(summary);
    preview.appendChild(tree);
    if (conflictLines.length > 0) {
      var warn = document.createElement("p");
      warn.textContent = "Target company has existing data; choose overwrite to replace.";
      preview.appendChild(warn);
    }

    var applyButton = document.createElement("button");
    applyButton.textContent = "Apply import (" + strategy + ")";
    applyButton.addEventListener("click", async function () {
      var applyResponse = await fetch(
        "/api/companies/" + companyId + "/import/archive?strategy=" + strategy,
        { method: "POST", headers: { "Content-Type": "application/octet-stream" }, body: body }
      );
      if (!applyResponse.ok) {
        preview.textContent = "Apply error " + applyResponse.status;
        return;
      }
      var result = await applyResponse.json();
      var s = result.summary || {};
      preview.textContent = "Imported " + s.imported + ", skipped " + s.skipped +
        ", failed " + s.failed + " | attachments restored: " + result.attachmentsRestored;
    });
    preview.innerHTML = "";
    preview.appendChild(applyButton);
  });
});
