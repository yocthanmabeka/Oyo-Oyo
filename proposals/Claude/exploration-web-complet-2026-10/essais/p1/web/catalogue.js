// catalogue.js — ce que le moteur HoloCode ferait seul est écrit ici à la main.
export const fold = (t) => t.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
const FIELDS = { id: "", title: "", theme: "", price: 0, image: "vide.svg", alt: "" };
// Comme l'arbitre : 100 éléments au plus, les champs déclarés seulement, un prix entier, une image du dossier.
export function clean(json) {
  const list = Array.isArray(json?.articles) ? json.articles.slice(0, 100) : [];
  return list.map((a) => Object.fromEntries(Object.entries(FIELDS).map(([k, d]) => {
    const v = a?.[k];
    if (typeof d === "number") return [k, Number.isInteger(v) && v >= 0 && v <= 1e9 ? v : ""];
    if (k === "image" && !(typeof v === "string" && /^[\w.\/-]+$/.test(v) && !v.includes(".."))) return [k, d];
    return [k, typeof v === "string" ? v.slice(0, 200) : typeof v === "number" ? String(v) : ""];
  })));
}
export function compute(articles, search, theme) {
  const s = fold(search);
  return articles
    .map((a, rank) => ({ a, rank }))
    .filter(({ a }) => fold(a.title).includes(s) && (theme === "" || a.theme === theme))
    .sort((x, y) => (Number(x.a.price) || 0) - (Number(y.a.price) || 0) || x.rank - y.rank)
    .map(({ a }) => a);
}
export const sum = (list, field) => list.reduce((n, a) => Math.min(n + (Number(a[field]) || 0), 1e9), 0);
const format = new Intl.NumberFormat("fr-FR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
export const cents = (n) => (n === "" ? "" : format.format(n / 100));

if (typeof document !== "undefined") {
  const $ = (id) => document.getElementById(id);
  const state = { search: "", theme: "", more: 6, failed: false, articles: [], cart: [] };
  try { state.cart = clean({ articles: JSON.parse(localStorage.getItem("cart") ?? "[]") }).map(({ id, title, price }) => ({ id, title, price })); } catch {}
  const built = new Map(); // clé tirée du contenu → nœud de la ligne
  function card(a) {
    const box = document.createElement("div");
    box.className = "card";
    const img = Object.assign(document.createElement("img"), { src: a.image, alt: a.alt });
    const h2 = Object.assign(document.createElement("h2"), { textContent: a.title });
    const price = Object.assign(document.createElement("span"), { textContent: `${cents(a.price)} €` });
    const add = Object.assign(document.createElement("button"), { type: "button", textContent: "Ajouter au panier" });
    add.addEventListener("click", () => { state.cart.push({ id: a.id, title: a.title, price: a.price }); render(); });
    box.append(img, h2, price, add);
    return box;
  }
  function render() {
    const found = compute(state.articles, state.search, state.theme);
    $("count").textContent = `${found.length} créations sur ${state.articles.length}. Panier : ${state.cart.length} (${cents(sum(state.cart, "price"))} €).`;
    $("failed").hidden = !state.failed;
    const shown = found.slice(0, state.more);
    const grid = $("grid");
    const lines = shown.map((a, rank) => {
      const key = JSON.stringify(a) + "#" + shown.slice(0, rank).filter((b) => JSON.stringify(b) === JSON.stringify(a)).length;
      if (!built.has(key)) built.set(key, card(a));
      return built.get(key);
    });
    lines.forEach((line, rank) => { if (grid.children[rank] !== line) grid.insertBefore(line, grid.children[rank] ?? null); });
    while (grid.children.length > lines.length) grid.lastElementChild.remove();
    $("empty").hidden = found.length > 0;
    $("more").hidden = !(found.length > state.more);
    try { localStorage.setItem("cart", JSON.stringify(state.cart)); } catch {}
  }
  async function load() {
    try {
      const response = await fetch("catalogue.json", { cache: "no-store" });
      if (!response.ok) throw new Error(response.status);
      state.articles = clean(JSON.parse((await response.text()).slice(0, 65536)));
    } catch { state.failed = true; }
    render();
  }
  $("search").addEventListener("input", (e) => { state.search = e.target.value; render(); });
  for (const radio of document.querySelectorAll("input[name=theme]")) radio.addEventListener("change", (e) => { state.theme = e.target.value; render(); });
  $("all").addEventListener("click", () => { state.theme = ""; document.querySelectorAll("input[name=theme]").forEach((r) => (r.checked = false)); render(); });
  $("more").addEventListener("click", () => { state.more += 6; render(); });
  $("retry").addEventListener("click", () => { state.failed = false; load(); });
  render();
  load();
}
