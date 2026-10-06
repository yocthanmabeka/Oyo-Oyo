# ADR-044 — Lot 7 : une liste qui change pendant la visite

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : le grand tableau (« des listes de valeurs », à 50 %) ; `ADR-040` (la répétition d'une liste écrite dans le fichier) ; Yocthan, le 2026-10-06 : « Oui, travaille sur ce qui reste »
- Validation : validé par Yocthan le 2026-10-06, après l'avoir essayé : « valide le point 1, 2, j'ai testé et ça marche en tout cas, donc du coup il faut le valider »
- Projets affectés : HoloCode, HoloEngine

## Contexte

`Repeat` (`ADR-040`) répète une liste écrite dans le fichier. Il manquait une liste que le visiteur remplit : des tâches, des notes, une liste d'envies.

## Décision

1. **Une liste est une valeur de la page** : `State(tasks: [])`, ou avec des éléments de départ, `State(tasks: ["Pain"])`. Des textes, cent au plus, de deux cents caractères au plus.
2. **Trois demandes**, seulement dans une règle `On` (un geste du visiteur) : `tasks.push(task)` ajoute le texte d'une valeur de la page (un texte vide n'est pas ajouté) ; `tasks.remove(item)` retire l'élément de la ligne touchée ; `tasks.clear()` vide la liste.
3. **Un texte se vide ou se remplit** : `task.set("")`, `task.set("Bonjour")`, `task.set(autreTexte)`.
4. **`Repeat(over: tasks, children: [ … ], rules: [ … ])`** montre une ligne par élément. Dans la ligne, `{item}` est le texte de l'élément. Les règles de la répétition répondent au bouton de la ligne touchée ; `item` y désigne cet élément.
5. **Une liste se montre par son nombre** (`{tasks}`) et se compare (`If(tasks, is: 0, …)`). `keep: [tasks]` la garde d'une visite à l'autre.
6. **Ce qu'un visiteur écrit reste du texte** : il est posé dans la ligne après la fabrication, échappé ; il ne devient jamais une balise, du gras, ni une valeur montrée (`{tache}` écrit par le visiteur reste « {tache} »).

La page fabriquée d'avance montre la liste de départ ; quand la liste change, le moteur fabrique ses lignes à nouveau et la page les pose à la place des anciennes. À son arrivée, le moteur redessine aussi une liste gardée d'une visite précédente.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Ce que contient une liste | des objets à champs ; **des textes** | Des textes d'abord : ce que le visiteur écrit. Des objets viendront avec les listes reçues du serveur. |
| Retirer un élément | par son numéro ; par son texte ; **par la ligne touchée (`item`)** | La ligne touchée : rien à compter, et deux textes identiques ne se confondent pas. |
| Qui change une liste | les règles de temps aussi ; **seulement un geste** | Un geste : une liste ne se vide pas toute seule sans que le visiteur ait rien touché. |
| Défauts du web évités | `innerHTML` avec ce que le visiteur a écrit (une faille classique) ; une liste fabriquée par le script, invisible avant son arrivée | Le texte est toujours échappé ; la liste de départ est dans la page fabriquée d'avance. |

## Conséquences

- Restent à faire : des listes d'éléments à champs (un panier avec nom, prix et quantité par ligne) ; recevoir une liste du serveur (`Data`) ; une liste dans une `List` à puces (aujourd'hui, `Repeat(over:)` se pose dans une page, une `Column`, une `Row` ou une `Grid`).

## Critères de validation

- Leçon 68 ; test du moteur `listes.rs` ; dans Chrome : ajouter, refuser un texte vide, vider le champ, retirer la ligne du milieu, tout effacer, et retrouver la liste après avoir rechargé la page.
