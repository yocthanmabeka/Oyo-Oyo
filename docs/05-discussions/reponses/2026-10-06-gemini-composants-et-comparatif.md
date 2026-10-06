# Réponse de Gemini — les composants faits pour le web, et le comparatif (2026-10-06)

Réponse au prompt [`../prompts/2026-10-06-gemini-composants-et-comparatif.md`](../prompts/2026-10-06-gemini-composants-et-comparatif.md), collée par Yocthan le 2026-10-06, **après** que les six points ont été construits. Gemini n'a pas accès au dépôt : il répond à l'état décrit dans le prompt (le matin). Résumé fidèle ; la lecture de Claude est à la fin.

## Ce que dit Gemini

### Comment font les autres

- **Flutter** : l'arbre de widgets, le PascalCase, les paramètres par constructeur ; mais aucune séparation entre structure et apparence : restyler demande `ThemeData` ou des dizaines de paramètres de style.
- **React** : `children` et la composition ; mais aucun cadre pour le style (CSS Modules, Tailwind, styled-components).
- **Vue, Svelte 5** : props en entrée, événements en sortie (`emit`), emplacements (`slot`, snippets), CSS cloisonné par défaut ; mais restyler un enfant depuis le parent demande `:deep()`.
- **Web Components** : Shadow DOM, `<slot>`, `::part()`, variables CSS qui traversent la frontière ; mais verbeux, et le Shadow DOM complique l'accessibilité.
- **Ce que le web sait faire que Flutter ne sait pas** : séparer le rôle d'un composant de sa recette graphique ; une feuille de style extérieure l'adapte (sombre, téléphone, promotion) sans toucher au composant.

### Ce qu'un bon composant doit avoir, dans cet ordre

1. Des paramètres avec **valeur par défaut**.
2. Un **emplacement de contenu** (`slot`, `children`).
3. Des **signaux vers la page** : le bouton ne change pas la page en secret, il émet un signal que la page écoute.
4. Des **variables d'apparence** et une **classe à l'appel**.
5. Plus tard, un état interne (un accordéon).

### Le restylage

Variables internes (`--accent`), classe à l'appel (`ArticleCard.promo`), style qui redéfinit la variable : « supérieur à `::part()`, qui est une syntaxe de programmeur ».

### Critique de l'écriture candidate, et sa proposition

- `params: [title, price, image]` n'a pas de valeur par défaut ; on ne voit pas comment le bouton prévient la page ; `Part` est trop abstrait.

```text
Component(
  name: ArticleCard,
  params: [title, price: 0, image: "placeholder.svg"],
  emits: [add],
  children: [ Column.card(children: [ Image(source: image, alt: title), H3("{title}"), Text("{price} euros"), Button(name: BtnAdd, text: "Ajouter") ]) ],
  rules: [ On(BtnAdd.tap, emit: add) ],
)

ArticleCard(title: "Sunrise", price: 120, image: "sunrise.png", onAdd: panier.add(1))
ArticleCard.promo(title: "The river", price: 90, onAdd: panier.add(1))
```

- **Les pièges** : les noms internes d'un composant doivent être privés ; refuser les boucles de composants ; un style interne ne doit jamais déborder sur le site.

### Son tableau (sur l'état du matin)

| | TS | React | Angular | Vue | Svelte | Solid | Astro | Elm | Flutter | HoloCode |
|---|---|---|---|---|---|---|---|---|---|---|
| Total pondéré | 58,0 | 65,5 | 59,0 | 77,5 | **83,0** | 72,0 | 80,5 | 62,8 | 63,0 | **68,5** |

Divergences avec Claude : HoloCode un peu plus bas (« ce qu'on peut construire » à 40) ; Flutter et React plus bas (poids, accessibilité, complexité). Chiffres d'opinion cités sans lien précis.

### Le reste du plan

- **À faire** : listes à champs, la place restante (`expand: true`), l'essai au lecteur d'écran.
- **Prématuré** : un fichier de styles à part (« le style sous le bloc, dans le même fichier, est une force ») ; un inspecteur de valeurs (« plus tard »).
- **À ne pas faire** : découper le moteur pour un moteur léger (« un piège d'ingénierie pour un gain invisible »).

---

## Lecture de Claude

**Ce qui était déjà fait avant sa réponse, et qui va dans son sens** : le composant posé comme un bloc, sans `Use` ; le restylage par variables et classe à l'appel, exactement sa « combinaison gagnante » ; les boucles de composants refusées ; les listes à champs ; la place restante (`grow: 1`, son `expand: true`) ; l'accessibilité vérifiée par axe-core, l'essai humain restant à faire.

**Ce qui diverge de ce qui est construit, et à trancher par Yocthan** :

1. **Les valeurs par défaut des paramètres** : pas encore là. Bonne idée, utile tout de suite (une image facultative). Son écriture `params: [title, price: 0]` mélange des noms et des réglages dans une liste ; à vérifier dans le lecteur, sinon `params: [title, price, image], defaults: …`.
2. **Les signaux émis (`emits`, `onAdd: panier.add(1)`)** contre **le paramètre qui reçoit le nom d'une valeur** (`qty: sunrise`, construit) : sa voie découple mieux le composant de la page ; la nôtre ne demande rien de nouveau à l'arbitre. Les deux peuvent coexister : `onAdd:` serait une écriture de plus.
3. **Des noms internes privés** : construit autrement, à dessein : `Add` devient `AddSunrise`, que la page peut écouter. Les rendre privés supposerait les signaux émis (point 2).
4. **`Component` plutôt que `Part`** : le mot est plus parlant, mais `Part` est décidé (`ADR-029`, `ADR-047`), employé dans le site de référence et les leçons ; le changer coûte une migration. À décider.
5. **Le style qui déborde** : même remarque que Codex ; décidé le 2026-10-06 : deux fichiers importés en conflit sont refusés, et le style par le nom du composant ne déborde jamais.
6. **Le moteur léger « à ne pas faire »** et **le fichier de styles « prématuré »** : déjà faits et fusionnés. Le moteur léger est quatre fois plus petit (149 Ko au lieu de 626), mesuré, sans coût pour l'auteur ; le fichier de styles est facultatif, le style sous le bloc reste la règle. Rien à défaire.
7. **Son tableau** : 68,5 % pour HoloCode le matin (Claude : 70 ; Codex : 74,6 avec les composants). Les trois tiennent Svelte en tête (Claude 82, Codex 85,4, Gemini 83).
