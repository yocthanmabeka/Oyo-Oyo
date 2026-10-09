# ADR-077 — Lot 9, premier pas : des modules qui reçoivent et rendent des textes, des listes et des nombres à virgule

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, lot 9 ; le point 1 des trois avis : « élargir les modules enfermés […] textes, listes et éléments à champs, toujours bornés ») ; `ADR-045` (« restent à faire : des échanges plus riches ») ; la revue de Codex (`proposals/GPT5.6/tout-le-web-avant-3d-2026-10-07/`, « format de module versionné »)
- Validation : Yocthan, le 2026-10-07, en confiant le lot 9 à la session du nuage : « que toi tu fasses le lot auquel il n'a pas encore touché » ; vérifié dans Chrome avant la fusion
- Projets affectés : HoloCode, HoloEngine

## Décision

```holo
module "97-bulletin.wasm"
Page(
  state: State(notes: [ Item(matiere: "Maths", note: "15.5") ], moyenne: 0.0, meilleure: "", nombre: 0),
  modules: [ Module(name: Bulletin, source: "97-bulletin.wasm", input: [notes], output: [moyenne, meilleure, nombre]) ],
  …
  rules: [ On(Calculer.tap, effect: Bulletin.run), On(Bulletin.done, effect: …), On(Bulletin.failed, effect: …) ],
)
```

1. **`input:` et `output:` acceptent un nom ou une liste de noms** : des nombres, à virgule aussi, des textes, des listes (de textes ou à champs) que la page déclare, seize au plus. Un module n'écrit ni l'heure, ni la date du visiteur, ni une liste calculée ; il peut en lire.
2. **Deux contrats**, reconnus à ce que le module offre :
   - le premier (`ADR-045`, inchangé) : `run(nombre) -> nombre`. Il suffit quand le module reçoit au plus un nombre et en rend un ;
   - le second : le module offre `alloc(taille) -> adresse` (où le moteur écrit ce qu'il reçoit ; 0 s'il n'a pas la place) et `run(adresse, taille) -> u64` (l'adresse de sa réponse dans les 32 bits du haut, sa taille dans ceux du bas).
3. **Ce qui voyage est un texte JSON** : `{"notes":[{"matiere":"Maths","note":"15.5"}]}`. Un nombre à virgule est écrit avec ses chiffres (`12.50`), un texte entre guillemets, échappé ; les champs d'un élément arrivent en texte, comme dans la page.
4. **La réponse est relue avec méfiance**, comme des données venues d'un serveur (`ADR-064`) :
   - un objet JSON de 64 Ko au plus ;
   - chaque clé est une valeur annoncée dans `output`, de la bonne sorte : un nombre positif pour un nombre (à virgule seulement dans une valeur à virgule), un texte pour un texte, un tableau pour une liste ;
   - sinon, toute la réponse est refusée, rien ne change, et le module a échoué (`Nom.failed`) ;
   - acceptée, elle est rangée dans les bornes de chaque valeur, puis `Nom.done`.
5. **La boîte ne change pas** : un fil à part, une mémoire donnée et plafonnée, ni réseau, ni page, ni heure ; arrêté au-delà de son temps.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Comment passer des textes et des listes | des nombres rangés en mémoire à des places convenues ; un format binaire (MessagePack) ; **un texte JSON** | lisible, déjà relu par le moteur pour `Data` ; n'importe quel langage sait l'écrire |
| Reconnaître le contrat | un réglage `contract: 2` dans le fichier ; **ce que le module offre** (`alloc`, et `run` à deux nombres) | rien de plus à écrire pour l'auteur ; un module du premier contrat marche toujours |
| Une clé non annoncée dans la réponse | l'ignorer ; **tout refuser** | un module qui rend autre chose que prévu est un défaut, ou une ruse : mieux vaut le savoir que l'avaler en silence (`ADR-037`) |
| Les champs d'un élément | des nombres quand ils en ont l'air ; **toujours des textes** | sans deviner : la page les garde en texte, le module les reçoit tels quels |

## Ce qui n'est pas fait

- Un module qui dessine (une liste bornée d'ordres de dessin) : l'étape 4 du lot 9.
- Des droits déclarés (`allow:`), le jour où un module aura besoin d'autre chose que ses valeurs.
- La même boîte côté serveur (`holo serve`) et dans la vue en profondeur.

## Critères de validation

- Tests du moteur : ce qu'un module reçoit (une liste à champs, un texte échappé, un nombre à virgule) ; une bonne réponse rangée, puis `done` ; refusés : une clé non annoncée, un texte dans un nombre, un nombre négatif, une liste dans un texte, autre chose qu'un objet, plus de 64 Ko ; une liste rendue ; refusés à la lecture : une valeur inconnue, un nom deux fois, l'heure en sortie, des textes au lieu de noms.
- Dans Chrome (leçon 97) : « 3 notes ; moyenne : 13,5 ; la meilleure : Maths » ; une note de plus, avec des guillemets et des accolades dans la matière, « 4 notes ; moyenne : 14,6 » ; le module qui ment refusé (« admin », non annoncé), rien de changé ; la leçon 69, du premier contrat, rend toujours 5 050.
