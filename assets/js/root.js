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
  let progressBar = document.querySelector("progress");
  let output = document.querySelector("output");

  let submitButton = document.querySelector("[data-submit]");
  submitButton.disabled = true;

  progressBar.classList.remove("hidden");
  output.classList.add("hidden");

  let submitButtonValue = submitButton.value;
  submitButton.value = "…";

  document.querySelector("input[name=body]").select();

  let formData = new FormData(form);

  let result = await fetch(`/raw?body=${formData.get("body")}`, {
    method: "GET",
  });

  let text = await result.text();

  output.value = text;
  output.classList.remove("hidden");
  progressBar.classList.add("hidden");
  submitButton.value = submitButtonValue;

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
