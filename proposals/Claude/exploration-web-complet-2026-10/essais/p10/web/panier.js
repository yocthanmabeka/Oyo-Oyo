// panier.js — la même page que propose.holo, écrite en JavaScript.
const MAX = 1e9; // les bornes de l'arbitre : 0 à 1 000 000 000
const clamp = (n) => Math.max(0, Math.min(MAX, n));
export function totals(cart) {
  const units = clamp(cart.reduce((n, l) => n + l.qty, 0));
  const subtotal = clamp(cart.reduce((n, l) => n + l.price * l.qty, 0));
  const vat = clamp(Math.round((subtotal * 20) / 100)); // au centime le plus proche, 0,5 vers le haut
  return { units, subtotal, vat, total: clamp(subtotal + vat) };
}
export function addDays(now, days) {
  const d = new Date(now.getFullYear(), now.getMonth(), now.getDate()); // l'heure de l'appareil, à minuit
  d.setDate(d.getDate() + days);
  return d;
}
const money = new Intl.NumberFormat("fr-FR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
export const cents = (n) => money.format(n / 100);
export const longDate = (d) => d.toLocaleDateString("fr-FR", { weekday: "long", day: "numeric", month: "long", year: "numeric" });

if (typeof document !== "undefined") {
  const start = [
    { id: "barque", title: "La barque", price: 1999, qty: 1 },
    { id: "phare", title: "Le phare", price: 11000, qty: 1 },
  ];
  let cart = start;
  try {
    const kept = JSON.parse(localStorage.getItem("cart"));
    if (Array.isArray(kept)) cart = kept.slice(0, 100).map((l) => ({ id: String(l.id ?? ""), title: String(l.title ?? "").slice(0, 200), price: clamp(Number.isInteger(l.price) ? l.price : 0), qty: clamp(Number.isInteger(l.qty) ? l.qty : 0) }));
  } catch {}
  const lines = document.getElementById("lines");
  function line(l, rank) {
    const row = document.createElement("div");
    row.className = "row";
    row.append(Object.assign(document.createElement("span"), { textContent: `${l.title}, ${cents(l.price)} € HT : ${l.qty}` }));
    const button = (text, change) => {
      const b = Object.assign(document.createElement("button"), { type: "button", textContent: text });
      b.addEventListener("click", () => { cart = cart.map((x, i) => (i === rank ? { ...x, qty: clamp(x.qty + change) } : x)); render(); });
      return b;
    };
    if (l.qty > 0) row.append(button("Un de moins", -1));
    row.append(button("Un de plus", +1));
    return row;
  }
  const built = new Map();
  function render() {
    // Une ligne qui n'a pas changé garde son nœud (et le focus) ; une ligne changée est refaite.
    const nodes = cart.map((l, rank) => {
      const key = JSON.stringify(l) + "#" + rank;
      if (!built.has(key)) built.set(key, line(l, rank));
      return built.get(key);
    });
    nodes.forEach((n, i) => { if (lines.children[i] !== n) lines.insertBefore(n, lines.children[i] ?? null); });
    while (lines.children.length > nodes.length) lines.lastElementChild.remove();
    const t = totals(cart);
    const status = document.getElementById("status"); // n'annoncer que ce qui change
    const said = `${t.units} créations : ${cents(t.subtotal)} € HT, TVA ${cents(t.vat)} €, total ${cents(t.total)} € TTC.`;
    if (status.textContent !== said) status.textContent = said;
    document.getElementById("delivery").textContent = `Livraison prévue le ${longDate(addDays(new Date(), 3))}.`;
    try { localStorage.setItem("cart", JSON.stringify(cart)); } catch {}
  }
  render();
  // Comme la page HoloCode qui lit l'heure : se tenir à jour à chaque minute (minuit change la date).
  setInterval(render, 60_000);
}
