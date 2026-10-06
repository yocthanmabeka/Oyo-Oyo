Bonjour Codex. C'est Yocthan. Avant la 3D, je veux que la version web de HoloCode soit « pas parfaite, mais assez utilisable ». Tu as accès au dépôt `yocthanmabeka/Metaverse` : lis, lance, et réponds en français, en phrases simples. Ne fusionne rien et ne change aucun statut de décision : range ta réponse dans `proposals/GPT5.6/`, par une pull request.

# Ce que tu dois lire

1. `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md` (branche `discussion/comparatif-langages-web`, ou `main` si elle est fusionnée) : la comparaison de Claude avec TypeScript, React, Angular, Vue, Svelte, SolidJS, Astro, Elm et Flutter ; la même liste de tâches écrite dans les dix ; ses notes ; ce que HoloCode ne prend pas, et pourquoi ; ses six points pour faire mieux.
2. `docs/01-holocode/GUIDE.md` : § 5 (les styles), § 6 sexies (`import`, `Part`, `Use`), § 6 quaterdecies (`Repeat`), § 11 (ce qui n'existe pas encore).
3. `docs/02-gouvernance/adr/` : `ADR-029` (imports et morceaux), `ADR-037` (écriture des noms), `ADR-040` (`Repeat`), `ADR-041` (le CSS utile), `ADR-033` (page légère et moteur chargé au besoin), `ADR-045` (module enfermé).
4. Dans le moteur : `moteur/src/blocs.rs`, `plat.rs`, `styles.rs`, `repetition.rs`, `etat.rs`, et `moteur/web/page.html`, `page-moteur.js`.

# 1. Les composants, faits pour le web

Ce que je veux, dans mes mots : « J'aime le composant, c'est une notion de Flutter que j'adore. Mais il faut des composants **faits pour le web**. Avec Flutter, un composant ne se réutilise pas facilement deux fois avec une autre allure. Sur le web, grâce au CSS, on devrait pouvoir l'utiliser deux fois facilement, avec une autre apparence. Si on arrive à faire de bons composants, ce sera un vrai ajout au web actuel. »

Aujourd'hui, un `Part` n'a ni paramètres, ni valeurs, ni règles. Écriture candidate de Claude, **à critiquer, pas à approuver** :

```holo
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

ArticleCard(title: "Sunrise", price: 120, image: "sunrise.png"),
ArticleCard.promo(title: "Night", price: 60, image: "night.png"),

ArticleCard { --accent: #E9B44C; }
.promo { --accent: crimson; }
```

Je te demande :

1. **Dans le moteur**, que faut-il changer pour qu'un `Part` reçoive des paramètres, et pour qu'il s'écrive comme un bloc ordinaire (`ArticleCard(…)`) ? Où vivent aujourd'hui les morceaux (`blocs.rs`, `plat.rs`) ? Comment éviter qu'un paramètre soit confondu avec une valeur de `State` ou un champ de `item` dans un `Repeat` ?
2. **Le signal vers la page** : le bouton « Ajouter » d'une carte doit changer le panier de la page. `Repeat` renomme déjà `Add` en `AddSunrise`. Faut-il la même chose pour les composants, ou un signal du composant (`ArticleCard.add`) ? Propose une écriture et montre comment l'arbitre la vérifie.
3. **Les valeurs et règles propres** à un composant (un compteur dans chaque carte) : faut-il les permettre dès la première version ? Quel risque pour l'arbitre ?
4. **Le restylage de l'extérieur, sans sélecteur composé** : variables CSS exposées (`--accent`), nom à point sur l'appel (`ArticleCard.promo`), parties nommées visées de l'extérieur (comme `::part()` des Web Components), variantes déclarées. Lesquelles le moteur peut-il produire en vrai CSS, sans fuite d'un composant à l'autre (`@scope`, classes générées) ? Compare avec Flutter, React, Vue, Svelte, Angular et les Web Components.
5. **Un emplacement pour du contenu** (comme `children` ou `<slot>`) : utile dès la première version ?
6. **Les pièges** : un composant qui s'utilise lui-même, des composants emboîtés, collisions de noms entre fichiers importés, HTML et accessibilité produits, poids. Écris des sondes, comme dans tes revues précédentes.

# 2. Le même tableau que Claude, rempli par toi

Reprends le tableau du § 4 de `COMPARATIF-LANGAGES-WEB.md` (mêmes critères, mêmes poids) et **remplis-le sans t'aligner sur Claude**, pour les dix. Vérifie ses nombres de lignes, et les notes de HoloCode contre le code réel (par exemple : l'accessibilité est-elle vraiment à 80 ? le poids d'une page qui bouge est-il bien d'environ 560 Ko ?). Si tu peux lire les pages officielles de Stack Overflow 2025 et 2026 et de State of JS 2025, corrige ses chiffres d'opinion, qui viennent d'extraits de recherche.

# 3. Le reste du plan web

Les points 2 à 6 de Claude : listes à champs (aussi venues du serveur), disposition (largeur, place restante, plusieurs noms de style, fichier de styles), moteur allégé pour les pages qui bougent (l'arbitre seul, moins de 100 Ko : est-ce faisable avec le découpage actuel ?), outils (voir les valeurs, mise en forme, essais écrits), essai au lecteur d'écran. Qu'est-ce qui manque, qu'est-ce qui est inutile, dans quel ordre ?

Pour chaque point : sépare ce que tu as vérifié en lançant le code de ce que tu supposes. Termine par une liste classée, du plus important au moins important.
