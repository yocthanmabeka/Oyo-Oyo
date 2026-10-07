// Essai de la logique de la version web du panier (sans navigateur), avec des dates fixées.
import { totals, addDays, cents, longDate } from "./web/panier.js";

const check = (label, seen, expected) => console.log(`${seen === expected ? "OK   " : "ÉCHEC"} ${label} : ${seen}${seen === expected ? "" : `  (attendu : ${expected})`}`);
const cart = [{ title: "La barque", price: 1999, qty: 1 }, { title: "Le phare", price: 11000, qty: 1 }];
let t = totals(cart);
check("hors taxe", cents(t.subtotal), "129,99");
check("TVA 20 % au centime le plus proche (2 599,8)", cents(t.vat), "26,00");
check("TTC", cents(t.total), "155,99");
t = totals([{ price: 1999, qty: 1 }]);
check("19,99 € : TVA 3,998 € → 4,00 €", cents(t.vat), "4,00");
t = totals([{ price: 1999, qty: 3 }, { price: 11000, qty: 0 }]);
check("trois barques, plus de phare", `${t.units} / ${cents(t.subtotal)}`, "3 / 59,97");
check("mercredi 7 octobre 2026 + 3 jours", longDate(addDays(new Date(2026, 9, 7, 23, 59), 3)), "samedi 10 octobre 2026");
check("fin de mois : 30 octobre + 3", longDate(addDays(new Date(2026, 9, 30, 12), 3)), "lundi 2 novembre 2026");
check("année bissextile : 28 février 2028 + 1", longDate(addDays(new Date(2028, 1, 28, 8), 1)), "mardi 29 février 2028");
check("borne : un total énorme s'arrête à 1 000 000 000", String(totals([{ price: 999_999_999, qty: 3 }]).subtotal), "1000000000");
