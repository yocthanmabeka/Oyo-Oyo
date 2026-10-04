// La même boutique : le comportement.
// Tout ce que ce fichier fait à la main, le moteur HoloCode le fait à partir de boutique.holo.

"use strict";

// ---------------------------------------------------------------- ce que le point déclare

const workshop = {
  name: "Workshop",
  seed: 42n,
  brightness: 0.8,
  fragments: 6,
  color: "#E9B44C",
  palette: ["#E9B44C", "#245C45"],
  budget: 500_000, // 500 KB, en octets
  inside: [
    { source: "../easel.svg", weight: 1_000 }, // 1 KB
  ],
};

const storeroom = { name: "Storeroom", seed: 7n };

// ---------------------------------------------------------------- les graines
// Les mêmes calculs que le moteur (moteur/src/graine.rs), pour obtenir les mêmes mondes.
// JavaScript n'a pas d'entiers de 64 bits ordinaires : il faut BigInt et un masque.

const MASK = (1n << 64n) - 1n;

function mix(x) {
  x = (x + 0x9E3779B97F4A7C15n) & MASK;
  x = ((x ^ (x >> 30n)) * 0xBF58476D1CE4E5B9n) & MASK;
  x = ((x ^ (x >> 27n)) * 0x94D049BB133111EBn) & MASK;
  return x ^ (x >> 31n);
}

function childSeed(parent, index) {
  return mix(parent ^ mix(0x5EED000000000000n | BigInt(index)));
}

// Une couleur tirée d'une graine, quand l'auteur n'en impose pas.
function colorFromSeed(seed) {
  const hue = Number(mix(seed) % 360n);
  return `hsl(${hue}, 80%, 65%)`;
}

// ---------------------------------------------------------------- le budget
// À vérifier avant d'afficher quoi que ce soit.

function checkBudget(point) {
  const weight = point.inside.reduce((sum, item) => sum + item.weight, 0);
  if (weight > point.budget) {
    throw new Error(`${point.name}: content weighs ${weight} B, budget is ${point.budget} B`);
  }
}

// ---------------------------------------------------------------- l'affichage des points

function paintPoint(element, color, brightness = 1) {
  element.style.setProperty("--color", color);
  element.style.setProperty("--brightness", brightness);
}

function paintFragments(point, container) {
  container.replaceChildren();
  for (let i = 0; i < point.fragments; i++) {
    const seed = childSeed(point.seed, i);
    const child = document.createElement("div");
    child.className = "point";
    child.dataset.seed = seed.toString();
    // Sur un cercle : le vrai moteur les place sur une sphère.
    const angle = (2 * Math.PI * i) / point.fragments;
    child.style.left = `${50 + 40 * Math.cos(angle)}%`;
    child.style.top = `${50 + 40 * Math.sin(angle)}%`;
    paintPoint(child, point.palette[i % point.palette.length]);
    container.append(child);
  }
}

// ---------------------------------------------------------------- les capacités : enter, leave

const shop = document.getElementById("shop");
const world = document.getElementById("workshop");

function enter() {
  shop.hidden = true;
  world.hidden = false;
  document.title = "The workshop";
}

function leave() {
  world.hidden = true;
  shop.hidden = false;
  document.title = "My shop";
}

// ---------------------------------------------------------------- les règles
// En HoloCode : On(Open.tap, effect: Workshop.enter). Ici, des écouteurs d'événements.

const workshopPoint = document.getElementById("workshop-point");

document.getElementById("open").addEventListener("click", enter);
document.getElementById("back").addEventListener("click", leave);
workshopPoint.addEventListener("click", enter);
workshopPoint.addEventListener("keydown", (event) => {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    enter();
  }
});

// ---------------------------------------------------------------- le panier
// En HoloCode : state: State(cart: 0), « {cart} » dans un texte, et On(Add.tap, effect: cart.add(1)).
// Ici, une variable que tout le script peut modifier, et un affichage à remettre à jour à la
// main après chaque changement. L'oublier une seule fois, et l'écran ment.

let cart = 0;

function showCart() {
  for (const place of document.querySelectorAll(".cart-count")) {
    place.textContent = cart;
  }
}

document.getElementById("add").addEventListener("click", () => {
  cart += 1;
  showCart();
});
document.getElementById("remove").addEventListener("click", () => {
  cart = Math.max(0, cart - 1); // sans ce garde-fou, le panier devient négatif
  showCart();
});
document.getElementById("empty").addEventListener("click", () => {
  cart = 0;
  showCart();
});

// ---------------------------------------------------------------- le démarrage

checkBudget(workshop);
paintPoint(workshopPoint, workshop.color, workshop.brightness);
paintFragments(workshop, document.getElementById("fragments"));
paintPoint(document.getElementById("storeroom-point"), colorFromSeed(storeroom.seed));

// Pour vérifier que les graines sont celles du moteur (voir README).
if (typeof module !== "undefined") {
  module.exports = { mix, childSeed };
}
