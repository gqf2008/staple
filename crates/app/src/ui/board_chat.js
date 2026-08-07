// Board Concierge chat: posts to /api/board/chat/stream and renders the SSE
// stream as chat bubbles (user right / agent left) with a streaming cursor,
// a thinking indicator, and optional tool/stderr accordions (forward-compatible
// with structured events). No-op when the chat form is not present.
document.addEventListener("DOMContentLoaded", function () {
  var form = document.getElementById("chat-form");
  var log = document.getElementById("chat-log");
  if (!form || !log) return;

  function bubble(role, headerText) {
    var wrap = document.createElement("div");
    wrap.className = "chat-bubble chat-bubble-" + role;
    if (headerText) {
      var header = document.createElement("div");
      header.className = "chat-bubble-header";
      header.textContent = headerText;
      wrap.appendChild(header);
    }
    var body = document.createElement("div");
    body.className = "chat-bubble-body";
    wrap.appendChild(body);
    log.appendChild(wrap);
    log.scrollTop = log.scrollHeight;
    return body;
  }

  function thinking() {
    var el = document.createElement("div");
    el.className = "chat-thinking";
    el.innerHTML = "<span></span><span></span><span></span>";
    log.appendChild(el);
    log.scrollTop = log.scrollHeight;
    return el;
  }

  function cursor() {
    var el = document.createElement("span");
    el.className = "chat-cursor";
    return el;
  }

  function toolAccordion(title, bodyText, isStderr) {
    var tool = document.createElement("div");
    tool.className = "chat-tool" + (isStderr ? " chat-tool-stderr" : "");
    var header = document.createElement("div");
    header.className = "chat-tool-header";
    header.textContent = (isStderr ? "stderr · " : "tool · ") + title;
    var body = document.createElement("div");
    body.className = "chat-tool-body";
    body.textContent = bodyText;
    header.addEventListener("click", function () {
      tool.classList.toggle("open");
    });
    tool.appendChild(header);
    tool.appendChild(body);
    return tool;
  }

  form.addEventListener("submit", async function (event) {
    event.preventDefault();
    var input = form.querySelector('textarea[name="message"]');
    var companyInput = form.querySelector('input[name="company_id"]');
    var adapterInput = form.querySelector('select[name="adapter_type"]');
    var message = (input && input.value ? input.value : "").trim();
    if (!message) return;
    var companyId = companyInput && companyInput.value ? companyInput.value : "";

    var userBody = bubble("user", "You");
    userBody.textContent = message;

    var agentBody = bubble("agent", "Assistant");
    var thinkingEl = thinking();
    var cursorEl = cursor();
    agentBody.appendChild(cursorEl);
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
        if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
        agentBody.removeChild(cursorEl);
        agentBody.textContent = "Error " + response.status;
        return;
      }
      if (!response.body) {
        if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
        agentBody.removeChild(cursorEl);
        agentBody.textContent = "Streaming not supported";
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
          if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
          try {
            var obj = JSON.parse(dataLine);
            if (obj.type === "delta") {
              var text = document.createTextNode(obj.content || "");
              agentBody.insertBefore(text, cursorEl);
            } else if (obj.type === "tool") {
              agentBody.appendChild(toolAccordion(obj.name || "call", obj.content || "", false));
            } else if (obj.type === "stderr") {
              agentBody.appendChild(toolAccordion(obj.name || "stderr", obj.content || "", true));
            } else if (obj.type === "done") {
              // keep the cursor hidden after completion
            } else if (obj.type === "error") {
              agentBody.insertBefore(document.createTextNode(" [error] "), cursorEl);
            }
          } catch (e) {
            agentBody.insertBefore(document.createTextNode(dataLine), cursorEl);
          }
          log.scrollTop = log.scrollHeight;
        }
      }
    } catch (err) {
      if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
      agentBody.removeChild(cursorEl);
      agentBody.textContent = "Error: " + err;
      return;
    }
    if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
    if (cursorEl.parentNode) cursorEl.parentNode.removeChild(cursorEl);
    log.scrollTop = log.scrollHeight;
  });
});
