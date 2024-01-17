document.addEventListener("DOMContentLoaded", function () {
  document.querySelectorAll("nav a").forEach(function (link) {
    if (
      (link.innerText === "home" && window.location.pathname === "/") ||
      window.location.pathname.endsWith(link.innerText)
    ) {
      link.classList.add("active");
    }
  });
});
