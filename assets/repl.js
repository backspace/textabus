document.addEventListener("DOMContentLoaded", function () {
  let form = document.querySelector("[data-form]");

  form.addEventListener("submit", async function (e) {
    let submitButton = document.querySelector("[data-submit]");
    submitButton.disabled = true;

    e.preventDefault();
    let formData = new FormData(form);

    let result = await fetch(`/raw?body=${formData.get("body")}`, {
      method: "GET",
    });

    let text = await result.text();

    document.querySelector("textarea").value = text;

    submitButton.disabled = false;
  });
});
