// Essai de la logique de la version web (sans navigateur) : les résultats attendus de la recette.
import { readFileSync } from "node:fs";
import { clean, compute, sum, cents } from "./web/catalogue.js";

const articles = clean(JSON.parse(readFileSync(new URL("./catalogue.json", import.meta.url), "utf8")));
const titles = (list) => list.map((a) => a.title).join(" | ");
const check = (label, seen, expected) => console.log(`${seen === expected ? "OK   " : "ÉCHEC"} ${label} : ${seen}${seen === expected ? "" : `  (attendu : ${expected})`}`);

check("12 articles reçus", String(articles.length), "12");
check("« FLEUVE »", titles(compute(articles, "FLEUVE", "")), "Lever sur le fleuve");
check("« aout » trouve « août »", titles(compute(articles, "aout", "")), "Nuit d'août");
check("thème Eau, prix croissant", titles(compute(articles, "", "Eau")), "La barque | Le phare | Lever sur le fleuve");
check("« zzz » : aucun résultat", String(compute(articles, "zzz", "").length), "0");
check("tous, triés : les six premiers", titles(compute(articles, "", "").slice(0, 6)), "La graine | Jour de marché | Dunes | Le verger | La barque | Le jardin d'hiver");
const cart = [articles[10], articles[10]];
check("deux fois La barque : total", cents(sum(cart, "price")), "150,00");
check("séparateur des milliers (U+202F)", JSON.stringify(cents(123450)), JSON.stringify("1 234,50"));
const odd = clean({ articles: [{ title: "Virgule", price: 12.5 }, { title: "Moins", price: -300 }, { title: "<b>x</b>", price: 100, image: "../secret.png" }] });
check("prix à virgule ou négatif : vide, comme l'arbitre", odd.map((a) => `${a.title}=${JSON.stringify(a.price)}`).join(" "), 'Virgule="" Moins="" <b>x</b>=100');
check("image hors du dossier refusée", odd[2].image, "vide.svg");
check("150 reçus, 100 gardés", String(clean({ articles: Array.from({ length: 150 }, (_, i) => ({ title: "t" + i, price: i })) }).length), "100");
