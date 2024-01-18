document.addEventListener("DOMContentLoaded", function () {
  document.querySelectorAll("tbody tr").forEach(function (row) {
    if (row.dataset.hasOwnProperty("unapproved")) {
      insertButton(row, "Approve", true);
    } else {
      insertButton(row, "Unapprove", false);
    }
  });
});

function updateApproval(number, approvedArg) {
  let path = approvedArg === "true" ? "approve" : "unapprove";

  fetch(`${window.location.href}/${number}/${path}`, {
    method: "POST",
  }).then(() => {
    window.location.reload();
  });
}

function insertButton(row, label, approvedArg) {
  row.insertAdjacentHTML(
    "beforeend",
    `<td><button onclick="updateApproval('${row.dataset.number}', '${approvedArg}')">${label}</button></td>`,
  );
}
