# ADR-027 — La saisie (`Input`, `Checkbox`) et les valeurs gardées (`keep`)

- Statut : ACCEPTÉ
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; `docs/01-holocode/COMPARATIF-CONCURRENTS.md` (planning, étape 3)
- Validation : Yocthan, le 2026-10-04 : « Tu corriges, ensuite tu fais l'étape 3. » L'écriture est une proposition de Claude ; à juger après essai. Validé par Yocthan le 2026-10-06 : « Qu'est-ce que tu attends pour valider tous ceux qui sont à l'essai ? »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Étape 3 du planning : les champs de saisie et les formulaires, et une valeur gardée après un rechargement. Jusqu'ici le visiteur ne pouvait qu'appuyer sur des boutons, et tout était perdu en rechargeant la page.

## Décision

```holo
Page(
  title: "My shop",
  state: State(cart: 0, gift: 0, tip: 0),
  keep: [cart, gift, tip],
  children: [
    Button(name: Add, text: "Add a painting"),
    Checkbox(value: gift, label: "Gift wrap"),
    Input(value: tip, label: "A tip, in euros", max: 50),
    If(gift, is: 1, children: [ "We will wrap your paintings." ]),
    Text("{cart} paintings, tip: {tip} euros"),
  ],
  rules: [
    On(Add.tap, effect: cart.add(1)),
  ],
)
```

1. **`Checkbox(value: gift, label: "…")`** : une case à cocher. Cochée, la valeur vaut 1 ; sinon 0.
2. **`Input(value: tip, label: "…", max: 50)`** : un champ où l'on écrit un nombre entier. `max` borne ce qu'on peut écrire.
3. **`value`** nomme une valeur de `State`. Le champ la montre, et la change quand le visiteur écrit. Aucune règle à écrire.
4. **`label` est obligatoire** : c'est lui qui dit ce qu'on attend, à l'œil comme à un lecteur d'écran.
5. **C'est encore l'arbitre qui change la valeur** (`etat::saisir`) : seulement une valeur qu'un champ présente, jamais au-delà de son plafond, et un texte qui n'est pas un nombre ne change rien.
6. **`keep: [cart, gift, tip]`**, sur la page : ces valeurs sont gardées dans le navigateur du visiteur. Il recharge la page, ou revient demain : elles sont encore là. Les autres repartent de leur départ.
7. Ce qui est relu du navigateur est tenu pour suspect : seules les valeurs nommées par `keep` sont reprises, et dans leurs bornes.

## Ajout du même jour : les valeurs de texte

Yocthan : « Tu termines l'étape 3. » Il manquait le champ de texte, et il lui fallait des valeurs qui soient du texte.

- `State(buyer: "")` déclare un texte. `{buyer}` le montre.
- `Input(value: buyer, label: "…", max: 20)` : le même bloc ; comme la valeur est un texte, le champ est un champ de texte, et `max` borne sa longueur (80 sans rien écrire, 200 au plus).
- Un texte ne se compare qu'au vide : `If(buyer, is: "")`, `If(buyer, not: "")`.
- Un texte ne change que par un champ. Il se garde par `keep`, comme un nombre.
- Ce que le visiteur écrit est nettoyé (pas de caractère invisible), borné, et montré lettre pour lettre : il ne devient jamais du code. Vérifié avec « Zoé <b>&; Arc ».

Choix faits : un seul bloc `Input` pour le nombre et le texte, plutôt que deux mots (en HTML, `input type=` a vingt-deux variantes) ; pas de comparaison de textes entre eux, qui appellerait vite « contient », « commence par », et des expressions.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Relier un champ à une valeur | `value: tip` (lié dans les deux sens) ; un signal `change` et une règle à écrire | `value:`. Écrire une règle pour dire « ce que je tape va dans la valeur » est du bruit ; c'est le défaut de React, que Vue et Svelte ont corrigé (`v-model`, `bind:value`). |
| L'étiquette | obligatoire ; facultative | Obligatoire. En HTML, un champ sans `label` est permis, et c'est l'une des fautes d'accessibilité les plus courantes. |
| Les mots | `Input`, `Checkbox` (HTML, Flutter) ; `Field`, `Check` | `Input` et `Checkbox`, connus de tous ceux qui ont fait du web. |
| Garder : où le dire | `keep: [noms]` sur la page ; un réglage `save: true` pour tout ; un second bloc `Saved(...)` | `keep: [noms]`. On choisit valeur par valeur : le panier oui, le temps d'une partie non. Et tout reste déclaré au même endroit. |
| Garder : où | le navigateur du visiteur ; un serveur | Le navigateur. Un serveur viendra avec l'étape 5. |

Défauts du web évités : en JavaScript, `localStorage` s'écrit et se relit à la main, sans vérifier ce qu'on relit ; un champ `type="number"` accepte quand même du texte collé, et la valeur dépasse `max` si le script ne vérifie pas. Ici, l'arbitre borne tout.

## Conséquences

### Positives

- Le panier de la boutique survit à un rechargement.
- Premier vrai formulaire : une case, un champ, sans une règle à écrire.

### Négatives et risques

- Un texte ne se compare qu'au vide, et rien ne peut le changer hors d'un champ : pas encore de recherche ni de filtre.
- Pas de liste de choix, pas de bouton radio, pas d'envoi à un serveur.
- La page arrive du serveur avec les valeurs de départ, puis prend les valeurs gardées : on peut voir « 0 » un instant avant « 3 ».
- Ce qui est gardé reste sur cet appareil, dans ce navigateur. Effacer les données du navigateur l'efface.
- Rien ne dit au visiteur que la page garde quelque chose.

## Ce qui reste à faire

- La liste de choix.
- Dire au visiteur ce que la page garde, et lui laisser l'effacer.

## Critères de validation

- `exemples/boutique-comparee/boutique.holo` : cocher la case, écrire 999 dans le champ (il retient 50), recharger : tout est encore là.
- Tests du moteur : `etat.rs` (`un_champ_une_case_et_des_valeurs_gardees`).

## Conditions de réexamen

- Quand Yocthan aura essayé et jugé : `Input`, `Checkbox`, `value`, `label`, `max`, `keep`.
- Quand les valeurs de texte existeront.
