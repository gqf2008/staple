// Global UI feedback (issue #231): disables submit buttons with a spinner
// while a mutating form POST is in flight (native navigation), and
// auto-dismisses the server-rendered flash toast. Lightweight vanilla JS in
// the same style as board_chat.js / command_palette.js; no dependencies.
// Forms handled by fetch-based scripts opt out with data-no-feedback.
document.addEventListener("DOMContentLoaded", function () {
  function setBusy(button) {
    if (!button || button.disabled) return;
    button.disabled = true;
    button.classList.add("btn-loading");
    var spinner = document.createElement("span");
    spinner.className = "spinner";
    spinner.setAttribute("aria-hidden", "true");
    button.appendChild(spinner);
  }

  var forms = document.querySelectorAll('form[method="post"]');
  for (var i = 0; i < forms.length; i++) {
    var form = forms[i];
    if (form.hasAttribute("data-no-feedback")) continue;
    form.addEventListener("submit", function () {
      var active = document.activeElement;
      if (active && active.tagName === "BUTTON" && active.type === "submit") {
        setBusy(active);
      } else {
        var buttons = this.querySelectorAll('button[type="submit"]');
        for (var j = 0; j < buttons.length; j++) setBusy(buttons[j]);
      }
      this.setAttribute("aria-busy", "true");
    });
  }

  var toast = document.getElementById("flash-toast");
  if (toast) {
    setTimeout(function () {
      toast.classList.add("hide");
      setTimeout(function () { toast.hidden = true; }, 300);
    }, 4000);
  }
});
