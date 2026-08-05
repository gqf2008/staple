// Board Concierge chat: posts to /api/board/chat/stream and renders the SSE
// stream incrementally. No-op when the chat form is not present.
document.addEventListener("DOMContentLoaded", function () {
  var form = document.getElementById("chat-form");
  var log = document.getElementById("chat-log");
  if (!form || !log) return;

  form.addEventListener("submit", async function (event) {
    event.preventDefault();
    var input = form.querySelector('textarea[name="message"]');
    var companyInput = form.querySelector('input[name="company_id"]');
    var adapterInput = form.querySelector('select[name="adapter_type"]');
    var message = (input && input.value ? input.value : "").trim();
    if (!message) return;
    var companyId = companyInput && companyInput.value ? companyInput.value : "";

    var userDiv = document.createElement("div");
    userDiv.className = "chat-message chat-user";
    userDiv.textContent = "You: " + message;
    log.appendChild(userDiv);

    var assistantDiv = document.createElement("div");
    assistantDiv.className = "chat-message chat-assistant";
    log.appendChild(assistantDiv);
    if (input) input.value = "";

    try {
      var response = await fetch("/api/board/chat/stream", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          companyId: companyId,
          message: message,
          adapterType: adapterInput ? adapterInput.value : undefined
        })
      });
      if (!response.ok) {
        assistantDiv.textContent = "Error " + response.status;
        return;
      }
      if (!response.body) {
        assistantDiv.textContent = "Streaming not supported";
        return;
      }
      var reader = response.body.getReader();
      var decoder = new TextDecoder();
      var buffer = "";
      for (;;) {
        var result = await reader.read();
        if (result.done) break;
        buffer += decoder.decode(result.value, { stream: true });
        var parts = buffer.split("\n\n");
        buffer = parts.pop() || "";
        for (var part of parts) {
          var dataLine = null;
          var lines = part.split("\n");
          for (var i = 0; i < lines.length; i++) {
            if (lines[i].indexOf("data:") === 0) { dataLine = lines[i].slice(5).trim(); break; }
          }
          if (!dataLine) continue;
          try {
            var obj = JSON.parse(dataLine);
            if (obj.type === "delta") { assistantDiv.textContent += obj.content || ""; }
            else if (obj.type === "done") { assistantDiv.textContent += "\n"; }
            else if (obj.type === "error") { assistantDiv.textContent += " [error] "; }
          } catch (e) {
            assistantDiv.textContent += dataLine;
          }
        }
      }
    } catch (err) {
      assistantDiv.textContent = "Error: " + err;
    }
  });
});
