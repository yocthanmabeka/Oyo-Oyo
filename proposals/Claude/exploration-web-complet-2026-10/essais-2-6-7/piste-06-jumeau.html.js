
  const $ = (id) => document.getElementById(id);
  const say = (text) => { $("live").textContent = ""; setTimeout(() => ($("live").textContent = text), 50); };
  // La valeur gardée d'une visite à l'autre (keep: [asked]).
  let asked = 0;
  try { asked = Number(localStorage.getItem("questions:asked") ?? 0); } catch {}
  $("ask").hidden = asked === 1;

  // When(size, is: "Grand") et When(size, is: "Petit") : au moment où le choix change.
  for (const radio of document.querySelectorAll("input[name=size]")) {
    radio.addEventListener("change", () => {
      if (radio.value === "Grand" && $("note").hidden) { $("note").hidden = false; say($("note").textContent); }
      if (radio.value === "Petit") $("note").hidden = true;
    });
  }
  $("ask").addEventListener("click", () => $("news").showModal());
  $("ok").addEventListener("click", () => $("news").close());
  // News.closed : par « D'accord », par la croix ou par Échap.
  $("news").addEventListener("close", () => {
    asked = 1;
    try { localStorage.setItem("questions:asked", "1"); } catch {}
    $("ask").hidden = true;
    // Le bouton qui avait le focus a disparu : le focus va au titre, pas au vide.
    document.querySelector("h1").focus();
  });
