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
    for (var i = 0; i < 3; i++) {
      var dot = document.createElement("span");
      el.appendChild(dot);
    }
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
    header.setAttribute("role", "button");
    header.setAttribute("tabindex", "0");
    header.textContent = (isStderr ? "stderr · " : "tool · ") + title;
    var body = document.createElement("div");
    body.className = "chat-tool-body";
    body.textContent = bodyText;
    function toggle() {
      tool.classList.toggle("open");
      header.setAttribute("aria-expanded", tool.classList.contains("open") ? "true" : "false");
    }
    header.addEventListener("click", toggle);
    header.addEventListener("keydown", function (event) {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        toggle();
      }
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

    var userLabel = form.getAttribute("data-user-label") || "You";
    var assistantLabel = form.getAttribute("data-assistant-label") || "Assistant";
    var userBody = bubble("user", userLabel);
    userBody.textContent = message;

    var agentBody = bubble("agent", assistantLabel);
    var thinkingEl = thinking();
    var cursorEl = cursor();
    agentBody.appendChild(cursorEl);
    if (input) input.value = "";

    try {
      var controller = typeof AbortController !== "undefined" ? new AbortController() : null;
      var timeoutId = controller ? setTimeout(function () { controller.abort(); }, 120000) : null;
      var response = await fetch("/api/board/chat/stream", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          companyId: companyId,
          message: message,
          adapterType: adapterInput ? adapterInput.value : undefined
        }),
        signal: controller ? controller.signal : undefined
      });
      if (!response.ok) {
        if (timeoutId) clearTimeout(timeoutId);
        if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
        if (cursorEl.parentNode) cursorEl.parentNode.removeChild(cursorEl);
        agentBody.textContent = "Error " + response.status;
        return;
      }
      if (!response.body) {
        if (timeoutId) clearTimeout(timeoutId);
        if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
        if (cursorEl.parentNode) cursorEl.parentNode.removeChild(cursorEl);
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
              agentBody.insertBefore(toolAccordion(obj.name || "call", obj.content || "", false), cursorEl);
            } else if (obj.type === "stderr") {
              agentBody.insertBefore(toolAccordion(obj.name || "stderr", obj.content || "", true), cursorEl);
            } else if (obj.type === "done") {
              if (cursorEl.parentNode) cursorEl.parentNode.removeChild(cursorEl);
            } else if (obj.type === "error" || obj.error !== undefined) {
              var errText = " [error" + (obj.error ? ": " + obj.error : "") + "] ";
              agentBody.insertBefore(document.createTextNode(errText), cursorEl);
            }
          } catch (e) {
            agentBody.insertBefore(document.createTextNode(dataLine), cursorEl);
          }
          log.scrollTop = log.scrollHeight;
        }
      }
    } catch (err) {
      if (timeoutId) clearTimeout(timeoutId);
      if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
      if (cursorEl.parentNode) cursorEl.parentNode.removeChild(cursorEl);
      agentBody.textContent = (controller && err && err.name === "AbortError")
        ? "[timeout]"
        : "Error: " + err;
      return;
    }
    if (timeoutId) clearTimeout(timeoutId);
    if (thinkingEl.parentNode) thinkingEl.parentNode.removeChild(thinkingEl);
    if (cursorEl.parentNode) cursorEl.parentNode.removeChild(cursorEl);
    log.scrollTop = log.scrollHeight;
  });
});
