// Pre-paint theme bootstrap (issue #242): applied synchronously in <head>
// so the first paint already has the correct `dark` class (no light flash).
(function () {
  try {
    var mode = localStorage.getItem("staple.theme") || "system";
    if (mode === "dark" || (mode === "system" && matchMedia("(prefers-color-scheme: dark)").matches)) {
      document.documentElement.classList.add("dark");
    }
  } catch (_) {}
})();
