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
    var previewPanel = document.createElement("div");
    previewPanel.id = "zip-file-preview";

    function renderContent(name, content) {
      if (!content || content.encoding !== "text") {
        previewPanel.textContent = content && content.encoding === "base64"
          ? "Binary file (" + (content.byteSize || 0) + " bytes) - not previewed."
          : "No preview available.";
        return;
      }
      var text = content.data || "";
      if (/\.md$/i.test(name) && text.indexOf("---") === 0) {
        var end = text.indexOf("\n---", 3);
        var header = end >= 0 ? text.slice(0, end + 5) : text;
        var body = end >= 0 ? text.slice(end + 5) : "";
        var table = document.createElement("table");
        header.split("\n").slice(1).forEach(function (line) {
          var idx = line.indexOf(":");
          if (idx <= 0) return;
          var row = document.createElement("tr");
          var key = document.createElement("td");
          key.textContent = line.slice(0, idx).trim();
          var value = document.createElement("td");
          value.textContent = line.slice(idx + 1).trim();
          row.appendChild(key);
          row.appendChild(value);
          table.appendChild(row);
        });
        previewPanel.innerHTML = "";
        if (table.children.length) previewPanel.appendChild(table);
        var pre = document.createElement("pre");
        pre.textContent = body.trim();
        previewPanel.appendChild(pre);
      } else {
        var pre = document.createElement("pre");
        pre.textContent = text;
        previewPanel.innerHTML = "";
        previewPanel.appendChild(pre);
      }
    }

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
          li.style.cursor = "pointer";
          li.addEventListener("click", function () {
            renderContent(node.name, node.content);
          });
        }
        parent.appendChild(li);
      });
    }
    renderTree(data.filesTree || [], tree);

    preview.innerHTML = "";
    preview.appendChild(summary);
    preview.appendChild(tree);
    preview.appendChild(previewPanel);
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
