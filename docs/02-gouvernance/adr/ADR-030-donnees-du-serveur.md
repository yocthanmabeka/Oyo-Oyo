# ADR-030 — Les données venues du serveur : `Data`

- Statut : ACCEPTÉ
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; `docs/01-holocode/COMPARATIF-CONCURRENTS.md` (planning, étape 5) ; revue de sécurité de Codex du 2026-10-03
- Validation : Yocthan, le 2026-10-04 : « Oui, fais le 5, le 6 et le 7 », puis « On y va ». L'écriture est une proposition de Claude ; à juger après essai. Validé par Yocthan le 2026-10-06 : « Qu'est-ce que tu attends pour valider tous ceux qui sont à l'essai ? »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Étape 5 du planning, seconde moitié. Une page ne savait montrer que ce qui était écrit dans son fichier. Un vrai site montre des choses qui changent sans qu'on récrive la page : un stock, un horaire, un message. C'est aussi la première fois qu'une page parle à un serveur après son ouverture : la revue de Codex a montré ce que cela peut coûter (requêtes vers d'autres serveurs sans geste du visiteur, fichiers sans limite).

## Décision (à l'essai)

```holo
Page(
  title: "My shop",
  state: State(stock: 0, message: ""),
  data: Data(from: "stock.json", every: 30s),
  children: [
    Text("{message}"),
    If(stock, over: 0, children: [ Text("{stock} paintings left.") ]),
    If(stock, is: 0, children: [ "Sold out." ]),
  ],
)
```

Le fichier `stock.json`, rangé à côté de la page :

```text
{ "stock": 4, "message": "Open until 6 pm" }
```

1. **`data: Data(from: "stock.json")`**, sur la page : elle va chercher ce fichier à l'ouverture.
2. **`every: 30s`** : elle le redemande à ce rythme, de `1s` à `3600s`. Sans `every`, une seule fois. Rien n'est demandé quand la fenêtre est cachée.
3. **Le fichier est un objet JSON à plat.** Chaque nom remplit la valeur de `State` du même nom : un nombre entier dans un nombre, un texte dans un texte, `true` et `false` dans un nombre (1 et 0).
4. **C'est l'arbitre qui range** (`etat::recevoir`) : seulement dans des valeurs déclarées, de la bonne sorte, et dans leurs bornes. Ce qui ne correspond à rien est laissé de côté. Un fichier mal formé, ou de plus de 64 Ko, ne change rien.
5. **La page ne parle qu'au serveur d'où elle vient** : `from` est un fichier `.json` rangé à côté, ni adresse complète ni remontée de dossier.
6. Les règles qui guettent (`When`) voient arriver les données, comme après un geste.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Ce que reçoit la page | des valeurs de `State` ; des morceaux de page (comme htmx) ; n'importe quel JSON, à parcourir | Des valeurs. Tout ce que le langage sait déjà faire avec une valeur (la montrer, la comparer, la guetter) vaut pour une donnée. Des morceaux de page feraient entrer du contenu que le vérificateur n'a pas vu. Parcourir un JSON demande un langage d'expressions. |
| Le format | un objet JSON à plat ; un fichier `.holo` de valeurs ; du texte `nom=valeur` | JSON. Tous les serveurs savent le produire. À plat : un objet emboîté n'aurait pas où aller. |
| D'où | à côté de la page seulement ; n'importe quelle adresse en `https` | À côté seulement, pour l'instant. Une page qui contacte d'elle-même un autre serveur, à l'ouverture et à intervalles, dit à ce serveur qui la lit et quand : c'est le pistage que la revue de Codex a fait retirer des passages. |
| Quand | à l'ouverture, puis à un rythme ; sur un geste ; en continu (WebSocket) | À l'ouverture, puis à un rythme borné. Le continu viendra avec le jeu à plusieurs (étape 7). |
| Quelles valeurs | celles dont le nom est dans le fichier ; une liste écrite dans la page | Celles du fichier. Une liste de plus à tenir serait du bruit. Limite : en lisant la page, on ne voit pas lesquelles viennent du serveur. |
| Le mot | `Data` ; `Fetch` ; `Source` | `Data`. `fetch` est un mot de programmeur, et `source` désigne déjà le fichier d'une image. |

Défauts de JavaScript évités : `fetch` vers n'importe où ; une réponse prise telle quelle et mise dans la page (`innerHTML`), donc du code ; pas de limite de taille ; une erreur de réseau qui casse la page ; un `setInterval` qui tourne dans un onglet caché.

## Conséquences

### Positives

- Un stock, un horaire, un message changent sans toucher au fichier de la page.
- Aucune surface nouvelle pour du code : ce qui arrive est un nombre ou un texte, rangé par l'arbitre et montré lettre pour lettre.

### Négatives et risques

- **Toujours pas de liste** : on ne peut pas recevoir « tous les articles du catalogue ». C'est la limite la plus gênante ; elle demande des valeurs qui soient des listes, et des blocs répétés.
- **La page ne sait qu'écouter** : elle n'envoie rien. Pas de commande, pas de message, pas d'inscription.
- Un autre serveur que le sien est exclu.
- La page arrive avec les valeurs de départ, puis prend les données : on peut voir « 0 » un instant.
- Le serveur de démonstration ne sert que des fichiers : pour voir une donnée changer, on modifie le fichier `.json` à la main.

## Ce qui reste à faire

- Les listes, et les blocs répétés.
- Envoyer quelque chose au serveur.
- Les données d'un autre serveur, avec l'accord du visiteur.

## Critères de validation

- `exemples/lecons/27-donnees.holo` : on change `27-donnees.json`, et en deux secondes la page suit.
- Tests du moteur : `etat.rs` (`les_donnees_d_un_serveur_passent_par_l_arbitre`).

## Conditions de réexamen

- Quand Yocthan aura essayé et jugé : `data`, `Data`, `from`, `every`.
- Quand Codex aura relu ce qui touche à la sécurité.
- Au premier site qui demande un catalogue.
