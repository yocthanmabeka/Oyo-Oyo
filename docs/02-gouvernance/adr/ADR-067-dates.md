# ADR-067 — Des dates : aujourd'hui, comparer, avancer, compter les jours

- Statut : PROPOSITION (construit et essayé ; attend la validation de Yocthan)
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07 (« terminer les données : … dates ») ; `ADR-042` (le champ date, `Input(type: date)`) ; `ADR-039` (l'heure du visiteur, `{year}` … `{minute}`).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Un champ date donnait un texte « 2026-10-06 », montré tel quel. On ne pouvait pas dire « 6 octobre 2026 », savoir si une date était passée, réserver « une semaine », ni compter les nuits entre une arrivée et un départ. Le champ acceptait même « 2026-13-45 », pourvu que les chiffres soient à leur place.

## Décision

1. **Une date reste un texte « AAAA-MM-JJ »**, la forme que donnent le champ date du navigateur et les données JSON. Une valeur est une date quand c'est `today`, quand un champ `Input(type: date)` la présente, ou quand elle est déclarée avec une date, `State(due: "2026-12-24")`.
2. **`today`** est la date du jour, donnée par l'appareil du visiteur (ou par le serveur qui fabrique la page), comme `hour` et `minute`. La page la tient à jour et passe minuit. Une règle qui la guette, `When(today, is: "2026-12-24", …)`, se déclenche ce jour-là. On ne la déclare pas et on ne la garde pas.
3. **L'affichage suit la langue de la page.**
   - `{arrival:date}` montre « 10 octobre 2026 » (« 1er octobre » le premier du mois), ou « October 10, 2026 » en anglais.
   - `{arrival:weekday}` montre « samedi ».
   - `{arrival}` reste « 2026-10-10 ».
4. **Deux dates se comparent dans le temps** : `If(departure, over: arrival)` (plus tard), `If(arrival, under: today)` (plus tôt), ou une date écrite, `If(arrival, over: "2026-12-24")`. Une date vide ne compare rien. Hors des dates, `over` et `under` restent pour les nombres (`ADR-063`).
5. **Une date avance ou recule de jours entiers** : `due.add(7)`, `due.sub(1)`, `due.set(today)`, `departure.set(arrival)`. Comme tout texte, seulement par un geste (`ADR-044`).
6. **Les jours entre deux dates** : `computed: [ Days(name: nights, from: arrival, to: departure) ]`.
   - `{nights}` montre le nombre, qui se refait à chaque changement.
   - Il se compare dans `If`, et sert dans une demande : `total.mul(nights)`.
   - Il vaut 0 si une date manque, ou si `to` vient avant `from` : pas encore de nombre négatif.
7. **Le champ date a des bornes** : `Input(type: date, min: today, max: "2026-12-31")`.
   - Le navigateur grise les jours hors bornes.
   - Le moteur refuse une date hors bornes, et aussi un jour qui n'existe pas au calendrier (« 2026-02-30 »).
8. **En chemin, `min:` vaut aussi pour un champ de nombre** : `Input(value: quantity, min: 1)`. Le moteur ne descend pas sous ce nombre. La longueur minimale d'un texte viendra avec les formulaires (lot 3), et elle est refusée avec cette raison.

## Comparaison faite avant de choisir

| Option | Écriture | Pour | Contre |
|---|---|---|---|
| **A. Un texte « AAAA-MM-JJ », que le moteur comprend** (proposée) | `If(arrival, under: today)`, `{arrival:date}`, `Days(…)` | la forme du champ date et du JSON : rien à convertir ; l'ordre des textes est l'ordre du temps | une date n'est pas un mot à part |
| B. Une valeur d'une sorte nouvelle | `State(arrival: Date(2026, 10, 10))` | une sorte bien à elle | un mot de plus ; des conversions au champ et aux données |
| C. Un nombre de jours | `State(arrival: 20736)` | les calculs d'un nombre | illisible pour qui écrit la page |

Défauts du web évités :

- `new Date("2026-10-10")` est lu en temps universel, et la veille s'affiche à Montréal ;
- les mois comptent de 0 en JavaScript ;
- une différence de dates passe par des millisecondes et se trompe au changement d'heure ;
- `toLocaleDateString` change selon le navigateur.

Ici, une date est un jour du calendrier, sans heure ni fuseau, et les calculs suivent le calendrier grégorien exact (années bissextiles comprises).

## Conséquences

- Le module `moteur/src/dates.rs` fait les calculs du calendrier (d'après les algorithmes de Howard Hinnant), vérifiés jour par jour de 1900 à 2100.
- `ADR-063` disait qu'`over` et `under` ne servaient qu'aux nombres : ils servent aussi à deux dates.
- Pas encore : une heure seule (« 14:30 ») ne se compare pas, une date ne se met pas dans une fiche de liste avec ses calculs, pas de nombre négatif de jours.

## Critères de validation

- Tests du moteur :
  - `the_calendar_is_exact` ;
  - `dates_are_compared_shifted_and_shown` : l'affichage, les bornes, les comparaisons, `add`, `sub`, `set(today)`, la règle qui guette, minuit, sept refus ;
  - `days_between_two_dates_count_nights` : 7 nuits, 80,00 € × 7 = 560,00 €, un départ avant l'arrivée, quatre refus.
- Dans Chrome, la leçon 87 :
  - « Nous sommes le … » à la date du jour ;
  - une arrivée dans trois jours, avec son jour de la semaine ;
  - « Une semaine » donne 7 nuits ;
  - 560,00 € ;
  - une arrivée hier est refusée.
