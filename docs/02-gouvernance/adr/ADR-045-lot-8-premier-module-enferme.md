# ADR-045 — Lot 8 : le premier module enfermé, et la preuve qu'on peut l'arrêter

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `ADR-011`, partie C (deux étages ; direction acceptée le 2026-10-06, « la construction commencera par la preuve demandée par Codex : arrêter un module qui boucle sans fin ») ; Yocthan, le 2026-10-06 : « Oui, travaille sur ce qui reste »
- Validation : validé par Yocthan le 2026-10-06, après l'avoir essayé : « valide le point 1, 2, j'ai testé et ça marche en tout cas, donc du coup il faut le valider »
- Projets affectés : HoloCode, HoloEngine

## Contexte

HoloCode refuse le code libre (`ADR-015`, `ADR-035`). Pour ce qu'il ne fait pas lui-même (un calcul lourd, une physique, une IA), la partie C d'`ADR-011` prévoit des modules compilés en WebAssembly, enfermés. Codex avait demandé de prouver d'abord qu'un module qui boucle sans fin peut être arrêté, avant de promettre une boîte fermée.

## Décision

```holo
module "somme.wasm"
Page(
  state: State(n: 100, total: 0),
  modules: [ Module(name: Sum, source: "somme.wasm", input: n, output: total, time: 100ms, memory: 1MB) ],
  children: [ Button(name: Go, text: "Compute"), P("{total}") ],
  rules: [ On(Go.tap, effect: Sum.run), On(Sum.done, effect: …), On(Sum.failed, effect: …) ],
)
```

1. **`module "somme.wasm"`** en haut du fichier : chaque module est annoncé, pour que le risque se lise d'un coup d'œil ; une annonce sans module, ou un module sans annonce, est refusé.
2. **`Module(name:, source:, input:, output:, time:, memory:)`**, déclaré sur la page (`modules:`, huit au plus). Il reçoit un nombre de la page (`input`) et en rend un (`output`). `time` va de 10ms à 5s (100ms sans rien dire) ; `memory` de 64KB à 16MB (1MB sans rien dire).
3. **Il se présente en holoscénique** : une capacité, `Sum.run`, et deux signaux, `Sum.done` (le nombre est arrivé dans `output`) et `Sum.failed` (il a été arrêté, ou il n'a pas pu démarrer).
4. **La boîte**, fabriquée par la page :
   - le module tourne dans un fil à part (un *worker*) : la page ne se bloque jamais ;
   - il ne reçoit que sa mémoire, plafonnée : elle ne peut pas grandir au-delà de `memory` ;
   - il n'a rien d'autre, ni réseau, ni page, ni heure, ni stockage : un module qui demande autre chose ne démarre pas ;
   - au-delà de son temps (compté à partir du moment où il commence), le fil est arrêté ; un démarrage de plus de 5 secondes aussi.
5. **Les ponts `bridge js` et `bridge css`**, rejetés (`ADR-011`, partie B), sont maintenant refusés à la lecture, avec un message qui mène aux modules.

## La preuve (dans Chrome, le 2026-10-06)

- Le module `compter` rend 5 050 pour n = 100.
- Le module `boucle` boucle sans fin : il est arrêté après 2 000 ms ; pendant qu'il tournait, trois touchers sur un autre bouton ont été comptés tout de suite.
- Le module `memoire` demande 64 Mo avec un plafond de 1 Mo : la mémoire ne grandit pas, il s'arrête.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où tourne le module | dans la page ; **dans un fil à part** | Seul un fil à part peut être arrêté sans arrêter la page. |
| La mémoire | celle que le module fabrique ; **celle que le moteur lui donne** (`--import-memory`) | Le plafond tient vraiment : la mémoire donnée ne grandit pas au-delà. |
| Ce qu'il échange | des textes, des listes ; **un nombre, et un nombre** | Le plus petit passage d'abord ; on l'élargira quand un vrai module en aura besoin. |
| Défauts du web évités | un script qui fige la page (« cette page ne répond pas ») ; un script tiers qui lit tout et parle à tout le monde | Le module ne peut ni figer la page, ni rien lire, ni rien envoyer. |

## Conséquences

- Restent à faire : des échanges plus riches (plusieurs nombres, un texte, une liste) ; des droits déclarés (`allow:`) le jour où un module aura besoin de quelque chose ; la même boîte dans la vue en profondeur et dans le futur navigateur propre au projet.
- Les modules de la leçon sont dans `exemples/lecons/modules/` (sources en Rust, `no_std`, une dizaine de lignes chacun).

## Critères de validation

- Leçon 69 ; test du moteur `modules.rs` ; l'épreuve dans Chrome décrite plus haut.
