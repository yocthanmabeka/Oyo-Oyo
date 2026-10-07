
  // url: [format] : le choix se lit dans l'adresse et s'y écrit (fiche.holo?format=Grand).
  const address = new URL(location.href);
  const chosen = address.searchParams.get("format") ?? "Grand";
  for (const radio of document.querySelectorAll("input[name=format]")) {
    radio.checked = radio.value === chosen;
    radio.addEventListener("change", () => {
      address.searchParams.set("format", radio.value);
      history.replaceState(history.state, "", address);
    });
  }
  // Data(from: "formats.json") remplit le tableau : chaque texte est échappé.
  const cents = new Intl.NumberFormat("fr-FR", { minimumFractionDigits: 2 });
  fetch("formats.json").then((r) => (r.ok ? r.json() : null)).then((data) => {
    if (!Array.isArray(data?.formats)) return; // un fichier mal écrit ne change rien
    const rows = data.formats.slice(0, 100).map((f) => {
      const tr = document.createElement("tr");
      const head = Object.assign(document.createElement("th"), { scope: "row", textContent: String(f.title ?? "") });
      const size = Object.assign(document.createElement("td"), { textContent: String(f.size ?? "") });
      const price = Object.assign(document.createElement("td"), { textContent: `${cents.format(Math.trunc(Number(f.price) || 0) / 100)} €` });
      tr.append(head, size, price);
      return tr;
    });
    document.getElementById("formats").replaceChildren(...rows);
  }).catch(() => { /* pas de réseau : la page garde ses lignes */ });
