Bonjour Gemini. C'est Yocthan Mabeka, pour le projet Holoverse / HoloCode. Tu n'as pas accès au dépôt : tout ce qu'il te faut est dans ce message. Réponds en français, en phrases simples. Cite tes sources, et dis clairement quand tu n'es pas sûr ou quand un chiffre est une estimation.

Avant la 3D, je veux que la version web de HoloCode soit « pas parfaite, mais assez utilisable ». Claude a comparé HoloCode à neuf langages et frameworks du web. Je te demande deux choses : **remplir le même tableau de ton côté**, et surtout **m'aider à concevoir les composants**, la notion que je préfère dans Flutter.

# Ce qu'est HoloCode aujourd'hui

- Un site s'écrit dans un fichier `.holo` : des blocs, des valeurs, des règles. **Jamais de code libre** (pas de fonction, pas de boucle, pas de `if` dans du code). Rien à installer. Le moteur (Rust, WebAssembly) fabrique du **vrai HTML et CSS** ; une page qu'on ne fait que lire pèse 8 Ko, le moteur (environ 560 Ko) ne se charge qu'au premier geste qui en a besoin.
- Les noms s'écrivent comme en Flutter : `PascalCase` pour les blocs (`Row`, `Column`, `Stack`, `BlueDoor`), `camelCase` pour les réglages, jamais `_`. Une faute est refusée avec le bon mot.
- Les styles s'écrivent comme du CSS, après le bloc racine. Un style vise **un type de bloc** (`P { … }`) ou **un nom à point** (`.card { … }`, posé en écrivant `P.card(…)`), pas plus : pas de sélecteur composé. Variables sans `var()` (`--gold`), états `hover:`, `focus:`, `active:`, `dark:`, `phone:`. Un style ne dit que l'apparence, jamais la disposition.

Exemple réel, une liste de tâches gardée d'une visite à l'autre (22 lignes) :

```holo
Page(
  title: "Une liste qui change",
  state: State(tache: "", taches: ["Encadrer le tableau de la rivière"]),
  keep: [taches],
  children: [
    H1("Une liste qui change"),
    Row(gap: 8px, children: [
      Input(value: tache, label: "Une tâche"),
      Button(name: Ajouter, text: "Ajouter"),
    ]),
    If(taches, is: 0,
      children: [ P("Rien à faire. Bravo !") ],
      else: [ P("{taches} tâche(s) à faire :") ],
    ),
    Repeat(over: taches, children: [
      Row(gap: 8px, children: [ Text("{item}"), Button(name: Fait, text: "Fait") ]),
    ], rules: [ On(Fait.tap, effect: taches.remove(item)) ]),
    Button(name: Vider, text: "Tout effacer"),
  ],
  rules: [
    On(Ajouter.tap, effect: [taches.push(tache), tache.set("")]),
    On(Vider.tap, effect: taches.clear()),
  ],
)

Page { background: #101020; color: white; }
.carte { border: 1px solid --gold; border-radius: 12px; hover: { box-shadow: 0 8px 24px #00000040; } }
```

**Les « morceaux » d'aujourd'hui, trop pauvres.** Un fichier commun peut définir `Part(name: Menu, children: [ … ])`, qu'une page pose avec `Use(Menu)`. Mais un morceau n'a **ni paramètres, ni valeurs, ni règles**. On ne peut pas écrire une « carte d'article » une fois et la poser dix fois avec un titre, un prix et une image différents. Seul `Repeat` répète un modèle, à l'intérieur d'une page.

# 1. Les composants, faits pour le web

Ce que je veux, dans mes mots : « J'aime le composant, c'est une notion de Flutter que j'adore. Mais il faut des composants **faits pour le web**. Avec Flutter, un composant ne se réutilise pas facilement deux fois avec une autre allure. Sur le web, grâce au CSS, on devrait pouvoir l'utiliser deux fois facilement, avec une autre apparence. Si on arrive à faire de bons composants, ce sera un vrai ajout au web actuel. »

Voici une écriture candidate de Claude, **à critiquer, pas à approuver** :

```holo
// Le composant, défini une fois (dans la page ou dans un fichier importé)
Part(
  name: ArticleCard,
  params: [title, price, image],
  children: [
    Column.card(children: [
      Image(source: image, alt: title),
      H3("{title}"),
      Text("{price} euros"),
      Button(name: Add, text: "Ajouter"),
    ]),
  ],
)

// Utilisé comme un bloc ordinaire, comme un widget Flutter
ArticleCard(title: "Sunrise", price: 120, image: "sunrise.png"),
ArticleCard(title: "The river", price: 90, image: "river.png"),
ArticleCard.promo(title: "Night", price: 60, image: "night.png"),

// Son apparence, et une variante d'un seul nom
ArticleCard { --accent: #E9B44C; }
.promo { --accent: crimson; }
```

Je te demande :

1. **Comment les autres le font**, avec ce qui marche et ce qui fâche : les widgets Flutter (et pourquoi les restyler est pénible : `ThemeData`, paramètres de style à passer à la main), les props et `children` de React, les props, slots et `emit` de Vue, les props et snippets de Svelte 5, les `input` / `output` d'Angular, et les **Web Components** (Shadow DOM, `<slot>`, `::part()`, propriétés personnalisées CSS), plus **`@scope`** en CSS. Qu'est-ce que le web sait faire que Flutter ne sait pas, pour l'apparence ?
2. **Ce qu'un bon composant web doit avoir**, et dans quel ordre le construire : des paramètres (avec une valeur par défaut ?), un emplacement pour du contenu (comme `children` ou `<slot>`), ses propres valeurs et règles, un signal vers la page (le bouton « Ajouter » de la carte doit pouvoir changer le panier de la page), des styles qui ne fuient pas, et **un restylage facile de l'extérieur**.
3. **Le restylage** : quelle est la façon la plus simple pour un débutant, sans sélecteur composé ? Des variables CSS que le composant expose (`--accent`) ? Un nom à point sur l'appel (`ArticleCard.promo`) ? Des parties nommées à l'intérieur, qu'on vise de l'extérieur (comme `::part()`) ? Des variantes déclarées dans le composant ?
4. **Critique de l'écriture candidate** : `params:` est-il le bon mot ? Faut-il écrire `ArticleCard(…)` directement, ou garder `Use(ArticleCard, …)` ? Comment un composant prévient-il la page qu'on a touché son bouton ? Propose ta propre écriture, la plus courte possible, sans code libre.
5. **Les pièges** : collisions de noms, composant dans un composant, boucles (un composant qui s'utilise lui-même), accessibilité (un composant doit produire du HTML correct), poids.

# 2. Le même tableau que Claude, rempli par toi

Note chaque technologie de 0 à 100 sur chaque critère (100 = l'excellence aujourd'hui), avec les poids pour le public de HoloCode (des gens qui ne programment pas, ou peu) :

| Critère | Poids |
|---|---|
| Facile pour un débutant | 20 |
| Court à écrire | 10 |
| Erreurs attrapées tôt, bien expliquées | 10 |
| Les valeurs qui changent (l'état) | 10 |
| Les composants réutilisables | 10 |
| Accessibilité et vrai HTML par défaut | 10 |
| Rapidité et poids | 10 |
| Ce qu'on peut construire | 10 |
| Les outils (éditeur, débogueur, tests) | 5 |
| Les bibliothèques, l'entraide | 5 |

Technologies : TypeScript, React, Angular, Vue, Svelte, SolidJS, Astro, Elm, Flutter, et HoloCode d'après ce message. Donne aussi les chiffres d'opinion que tu connais (Stack Overflow 2025 et 2026, State of JS 2025), **avec leur source**. Pour information, Claude trouve : Svelte 82 %, Vue 77 %, Astro 73 %, SolidJS 72 %, React 71 %, HoloCode 70 %, Flutter 70 %, Angular 67 %, Elm 62 %, TypeScript 59 % ; HoloCode est dernier sur les composants (45), ce qu'on peut construire (45), les outils (35) et l'entraide (5). **Fais ton tableau sans t'aligner sur le sien**, puis dis où vous divergez le plus.

# 3. Le reste du plan web

Claude propose, avant la 3D : (2) des listes d'éléments à champs, aussi reçues du serveur ; (3) la largeur d'un élément, un élément qui prend la place restante, plusieurs noms de style par bloc, un fichier de styles à part ; (4) un moteur allégé pour les pages qui bougent (l'arbitre seul, moins de 100 Ko) ; (5) des outils : voir les valeurs pendant l'essai, une mise en forme automatique, des essais écrits ; (6) un essai avec un lecteur d'écran. Qu'est-ce qui manque ? Qu'est-ce qui est inutile ?

Termine par un tableau : à faire maintenant, plus tard, à ne pas faire, avec une phrase de justification chacun.
