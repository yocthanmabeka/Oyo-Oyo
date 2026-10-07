
  const $ = (id) => document.getElementById(id);
  const form = $("booking");
  const say = (text) => { $("live").textContent = ""; setTimeout(() => ($("live").textContent = text), 50); };
  let key = crypto.randomUUID(); // une clé par réservation : un second envoi n'est pas un doublon
  let sending = false;

  // Le champ d'adresse n'existe que pour une livraison à domicile.
  form.addEventListener("change", (e) => {
    if (e.target.name === "delivery") $("address-row").hidden = e.target.value !== "À domicile";
  });

  // Vérifie un champ : le message sous le champ, relié à lui, ou rien.
  function check(field) {
    const box = field.type === "radio" ? $("delivery") : field;
    const id = `${box.id}-error`;
    let message = "";
    if (!field.closest("[hidden]")) {
      if (field.validity.valueMissing) message = "Ce champ est à remplir.";
      else if (field.validity.typeMismatch) message = "Un courriel s'écrit comme nom@exemple.fr.";
    }
    $(id)?.remove();
    box.removeAttribute("aria-invalid");
    box.removeAttribute("aria-describedby");
    if (message) {
      (field.type === "radio" ? box : field.parentElement).append(Object.assign(document.createElement("p"), { id, className: "error", textContent: message }));
      box.setAttribute("aria-invalid", "true");
      box.setAttribute("aria-describedby", id);
    }
    return !message;
  }
  const fields = () => [$("name"), $("mail"), form.querySelector("input[name=delivery]"), $("address")];
  // Une erreur s'efface dès que le champ devient juste.
  form.addEventListener("input", (e) => {
    const field = e.target.type === "radio" ? form.querySelector("input[name=delivery]") : e.target;
    if ((field.type === "radio" ? $("delivery") : field).hasAttribute("aria-invalid")) check(field);
  });

  form.addEventListener("submit", async (e) => {
    e.preventDefault(); // la page ne se recharge pas ; Entrée dans un champ arrive ici aussi
    if (sending) return;
    const wrong = fields().filter((f) => !check(f));
    if (wrong.length) return wrong[0].focus();
    sending = true;
    const button = form.querySelector("button");
    button.setAttribute("aria-disabled", "true");
    button.classList.add("waiting");
    $("failed").hidden = true;
    say("Envoi en cours…");
    const values = { name: $("name").value, mail: $("mail").value, delivery: form.delivery.value, gift: $("gift").checked ? 1 : 0, lever: 2, porte: 1 };
    if (!$("address-row").hidden) values.address = $("address").value;
    let arrived = false;
    try {
      const response = await fetch(location.pathname, {
        method: "POST",
        headers: { "content-type": "application/json", "idempotency-key": key },
        body: JSON.stringify({ form: "Booking", values }),
        signal: AbortSignal.timeout(15000),
      });
      arrived = response.ok;
    } catch { /* pas de réseau, serveur muet, ou plus de 15 s */ }
    sending = false;
    button.removeAttribute("aria-disabled");
    button.classList.remove("waiting");
    if (arrived) {
      key = crypto.randomUUID();
      $("sent").hidden = false;
      say($("sent").textContent);
    } else {
      $("failed").hidden = false;
      say($("failed").textContent);
    }
  });
