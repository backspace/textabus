let form;

document.addEventListener("DOMContentLoaded", function () {
  activateCommands();

  form = document.querySelector("[data-form]");

  form.addEventListener("submit", async function (e) {
    e.preventDefault();

    await submitForm();
  });
});

async function submitForm() {
  let submitButton = document.querySelector("[data-submit]");
  submitButton.disabled = true;

  let formData = new FormData(form);

  let result = await fetch(`/raw?body=${formData.get("body")}`, {
    method: "GET",
  });

  let text = await result.text();

  document.querySelector("textarea").value = text;

  submitButton.disabled = false;
}

function activateCommands() {
  document.querySelectorAll("[data-commands] li").forEach((element) => {
    let command = element.innerText.trim();
    let anchorElement = document.createElement("a");
    anchorElement.href = "#";

    anchorElement.addEventListener("click", executeCommand);
    anchorElement.appendChild(element.querySelector("code"));

    element.appendChild(anchorElement);
  });
}

function executeCommand(e) {
  e.preventDefault();

  let command = e.target.innerText.trim();

  document.querySelector("[data-body]").value = command;
  submitForm();
}
