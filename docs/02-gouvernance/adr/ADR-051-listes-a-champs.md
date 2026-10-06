# ADR-051 — Les listes à champs, aussi reçues du serveur

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md` (point 2 : « des listes à champs, dans la page et venues du serveur ») ; `ADR-044` (les listes de textes) ; `ADR-030` (`Data`) ; `ADR-040` (`Item`)
- Validation : Yocthan, le 2026-10-06 : « je suis tes propositions, je les valide, tu as mon feu vert ».
- Projets affectés : HoloCode, HoloEngine

## Contexte

Une liste ne contenait que des textes, et `Data` ne recevait que des nombres et des textes à plat : on ne pouvait pas montrer un catalogue venu du serveur, ni une liste d'articles avec un titre et un prix.

## Décision

1. **Un élément de liste peut avoir des champs**, écrits comme un `Item` : `State(articles: [ Item(title: "Sunrise", price: 12000) ])`. Tous les éléments ont les mêmes champs, dans le même ordre ; un champ est un texte (deux cents caractères au plus) ou un nombre entier ; seize champs au plus ; cent éléments au plus.
2. **Dans les lignes** de `Repeat(over: articles, …)` : `{item.title}`, `{item.price:cents}` (les formats d'`ADR-043`), et `item.image` à la place d'une valeur. Un champ inconnu est refusé, avec la liste des champs ; `{item}` montre le premier champ.
3. **`articles.push(Item(title: name, price: price))`** : chaque champ prend un texte, un nombre, ou le nom d'une valeur de la page ; un élément tout vide n'est pas ajouté. `remove(item)` et `clear()` restent.
4. **`Data` remplit une liste** : un tableau d'objets JSON remplit une liste à champs (seuls les champs déclarés sont repris ; ceux qui manquent valent "" ou 0), un tableau de textes une liste de textes. Une liste que la page ne déclare pas est laissée de côté.
5. `State(articles: [])` : une liste vide au départ peut recevoir des textes ou des éléments à champs ; ses champs ne sont alors pas vérifiés.
6. **Sûreté** : un élément à champs voyage dans l'état codé, précédé d'un signe que le visiteur ne peut pas écrire ; tout ce que le visiteur saisit et tout ce que le serveur envoie est échappé dans la page ; une image reçue reste dans le dossier de la page.

## Alternatives écartées

- **Des listes de listes, des objets emboîtés** : plus puissant, mais un débutant n'en a pas besoin pour un catalogue ; trois niveaux de JSON sont lus, le reste est ignoré.
- **Des nombres à virgule** : les prix s'écrivent en centimes et se montrent avec `{price:cents}` ; on évite les erreurs d'arrondi.

## Conséquences

- La leçon 71 (`exemples/lecons/71-liste-a-champs.holo`, `71-catalogue.json`) ; le guide (§ 6 septendecies, et `Data`).
- Pas encore de condition sur un champ dans une ligne (`If(item.done, …)`) ; à ajouter quand un exemple le demandera.

## Critères de validation

- Tests du moteur : une liste à champs montrée, formatée, remplie par un geste, vidée d'une ligne, remplie par le serveur (champs non déclarés ignorés, image hors du dossier refusée), une liste de textes reçue ; les refus.
- La leçon 71 dans Chrome : le catalogue arrive du fichier JSON ; « Retirer » et « Ajouter » marchent.
