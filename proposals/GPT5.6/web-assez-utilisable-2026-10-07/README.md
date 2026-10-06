# Revue GPT-5.6 — rendre le web HoloCode assez utilisable avant la 3D

- Auteur : GPT-5.6, pour Yocthan
- Date : 2026-10-07
- Statut de cette contribution : **PROPOSITION / REVUE**.
- Aucun statut d'ADR n'est modifié.
- Base lue : `main` au commit `dfb7c1ec6704c97505eac20e95205e0cec55b3a4`.
- Le comparatif demandé est déjà fusionné sur `main` et identique à la branche `discussion/comparatif-langages-web`.

## Réponse courte

La version web est déjà plus avancée que le prompt ne le suppose : les composants à paramètres existent maintenant et ADR-050 est accepté.

Je ne conseille donc pas de refaire les composants.

Je conseille de **consolider leur isolation CSS**, puis de faire les **listes à champs**, puis de **séparer le petit moteur interactif du moteur 3D**, puis les outils et l'essai lecteur d'écran.

Le plus gros manque actuel pour un vrai site n'est plus le composant. C'est la donnée structurée dynamique.

---

# 0. Ce que j'ai vérifié, ce que je n'ai pas pu relancer

## Vérifié directement dans le dépôt

J'ai lu :

- `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md`
- `docs/01-holocode/GUIDE.md`
- ADR-029, ADR-033, ADR-037, ADR-040, ADR-041, ADR-045, ADR-050
- `moteur/src/blocs.rs`
- `moteur/src/composants.rs`
- `moteur/src/plat.rs`
- `moteur/src/styles.rs`
- `moteur/src/repetition.rs`
- `moteur/src/listes.rs`
- `moteur/src/etat.rs`
- `moteur/src/holo.rs`
- `moteur/src/lib.rs`
- `moteur/Cargo.toml`
- `moteur/web/page.html`
- `moteur/web/page-moteur.js`
- `moteur/README.md`

Le code actuel contient déjà `moteur/src/composants.rs`.

Le GUIDE contient déjà « 6 sexies bis. Les composants ».

ADR-050 est déjà **ACCEPTÉ**. Cette revue ne change pas ce statut.

## Tentative de relance

J'ai tenté de cloner `https://github.com/yocthanmabeka/Metaverse.git` dans le runtime local pour relancer `cargo test`.

La commande a échoué avant le clonage :

```
Could not resolve host: github.com
```

Je n'annonce donc **aucun nouveau résultat d'exécution local**.

Les sondes ajoutées avec cette revue sont écrites pour être copiées dans `moteur/tests/` ou intégrées au harnais de revue. Elles n'ont pas été exécutées dans cette session.

Quand je dis « vérifié » ci-dessous, cela veut dire soit :

- vérifié par lecture directe du code actuel ;
- soit mesure déjà consignée dans le dépôt.

Quand je dis « estimation », cela reste à mesurer.

---

# 1. Les composants : état réel du moteur

## 1.1 Ce que le prompt demande existe déjà

Le moteur accepte aujourd'hui :

```holo
Page(
  state: State(cart: 0, sunrise: 0),
  parts: [
    Part(
      name: ArticleCard,
      params: [title, price, qty],
      children: [
        Column(children: [
          H2("{title}"),
          Text("{price:cents} euros"),
          Button(name: Add, text: "Add"),
        ]),
      ],
      rules: [
        On(Add.tap, effect: [qty.add(1), cart.add(price)]),
      ],
    ),
  ],
  children: [
    ArticleCard(name: Sunrise, title: "Sunrise", price: 12000, qty: sunrise),
  ],
)
```

Ce n'est plus une écriture candidate : c'est l'implémentation actuelle d'ADR-050.

### Vérifié dans le code

`composants.rs` :

- lit `Part(name:, params:, children:, rules:)` ;
- borne les paramètres à 16 ;
- exige un seul bloc racine ;
- autorise les règles dans le composant ;
- transforme un appel `ArticleCard(...)` en blocs ordinaires ;
- remplace les paramètres avant le reste des vérifications ;
- renomme les blocs internes ;
- interdit l'auto-récursion ;
- borne l'imbrication à 8 ;
- borne les copies à 2 000.

`holo.rs` :

- lit d'abord les composants importés ;
- ajoute ensuite les composants de la page ;
- refuse deux composants de même nom ;
- pose les composants **avant** de déplier les `Repeat`.

C'est le bon ordre.

---

# 2. Où vivent les morceaux aujourd'hui ?

Il y a maintenant deux mécanismes qui partent du même mot `Part`.

## Morceau simple

ADR-029 :

```holo
Part(
  name: Menu,
  children: [ ... ],
)
```

Puis :

```holo
Use(Menu)
```

Ce mécanisme reste utile pour :

- un menu ;
- un pied de page ;
- plusieurs blocs à insérer ensemble ;
- un morceau sans paramètres ni règles.

## Composant

ADR-050 :

```holo
Part(
  name: ArticleCard,
  params: [title],
  children: [ Column(...) ],
)
```

Puis :

```holo
ArticleCard(title: "Sunrise")
```

Dans le code :

- la lecture et l'assemblage des imports sont surtout dans `holo.rs` ;
- le sens du composant est dans `composants.rs` ;
- le HTML final est fabriqué dans `plat.rs`.

Je trouve cette séparation saine.

Je ne remettrais pas la logique des composants dans `plat.rs`.

Le rendu HTML ne doit pas connaître les paramètres : il doit recevoir des blocs déjà dépliés.

---

# 3. Paramètre, State et item : éviter les confusions

## Ce que fait le moteur aujourd'hui

Un paramètre ne peut pas porter le nom d'une valeur de la page.

Exemple refusé :

```holo
Page(
  state: State(title: ""),
  parts: [
    Part(
      name: Card,
      params: [title],
      ...
    ),
  ],
)
```

`item` est aussi réservé.

Les mots du langage sont réservés.

Dans un `Repeat`, les composants sont dépliés avant la répétition ; les expressions `item.title` restent donc reconnaissables par le dépliage de `Repeat`.

## Mon avis

**Garder cette règle pour V1.**

Elle est un peu restrictive, mais très lisible pour un non-programmeur.

Je ne ferais pas de portée lexicale avec priorité implicite du type :

> dans un composant, `title` veut dire le paramètre, sauf dans tel contexte où il veut dire `State.title`.

Ce serait plus puissant, mais moins prévisible.

La règle actuelle donne trois espaces clairs :

- `title` dans un composant : paramètre ;
- `cart` : valeur de page ;
- `item.title` : champ d'un élément de `Repeat`.

C'est plus simple à expliquer.

---

# 4. Le signal vers la page

## Ce que fait le moteur aujourd'hui

Dans une copie nommée :

```holo
ArticleCard(name: Sunrise, ...)
```

le bouton :

```holo
Button(name: Add, ...)
```

devient :

```
AddSunrise
```

Une règle de composant est clonée avec la copie.

Une règle de page peut aussi écouter :

```holo
On(AddSunrise.tap, ...)
```

Le test actuel de `composants.rs` vérifie ce comportement.

## Faut-il remplacer cela par `ArticleCard.add` ?

**Non, pas maintenant.**

`ArticleCard.add` ne dit pas quelle copie a émis le signal.

Avec deux cartes :

- Sunrise
- Night

on retombe immédiatement sur un second problème d'identité.

Il faudrait alors inventer :

- `Sunrise.add` ;
- ou un événement avec charge utile ;
- ou une remontée d'événement à la React/Vue ;
- ou une notion d'instance.

Tout cela ajoute une deuxième architecture d'événements.

Le système actuel a un avantage majeur : **après dépliage, l'arbitre ne sait même pas qu'il y avait un composant.**

C'est excellent.

## Écriture recommandée pour V1

Je garderais :

```holo
Part(
  name: ArticleCard,
  params: [price, qty],
  children: [
    Button(name: Add, text: "Add"),
  ],
  rules: [
    On(Add.tap, effect: [qty.add(1), cart.add(price)]),
  ],
)

ArticleCard(name: Sunrise, price: 12000, qty: sunrise)
```

Le moteur produit logiquement :

```holo
Button(name: AddSunrise, text: "Add")
On(AddSunrise.tap, effect: [sunrise.add(1), cart.add(12000)])
```

L'arbitre vérifie alors exactement les mêmes choses qu'avant :

1. le signal existe ;
2. la valeur ciblée existe ;
3. la demande est autorisée ;
4. la valeur fournie comme paramètre de mutation est bien une valeur de page.

Aucun arbitre de composant supplémentaire.

---

# 5. Valeurs propres à chaque composant

## Ma réponse

**Pas dans la première version.**

Les règles propres au composant : oui, elles existent déjà.

Un `State` interne à chaque copie : non.

## Pourquoi ?

Dès qu'une copie a son propre état, il faut définir :

- son identité stable ;
- ce qui arrive quand la copie disparaît ;
- ce qui arrive quand un `Repeat` change d'ordre ;
- comment `keep` la sauvegarde ;
- comment le serveur multijoueur la sérialise ;
- comment une copie importée change de version ;
- comment un état interne apparaît dans le débogueur ;
- comment on évite deux instances portant le même état.

Le système actuel évite tout cela :

```holo
ArticleCard(qty: sunrise)
```

La page reste propriétaire de l'état.

C'est plus verbeux pour 50 cartes, mais justement les **listes à champs** doivent régler ce cas.

Je ne résoudrais pas ce problème deux fois.

---

# 6. Restylage : comparaison avec le web

## Flutter

Flutter possède des widgets très réutilisables, mais l'apparence passe généralement par :

- des paramètres ;
- le `Theme` ;
- un thème local ;
- `copyWith`.

Le style n'est pas naturellement un mécanisme externe aussi libre que le CSS.

Documentation : https://docs.flutter.dev/cookbook/design/themes

## React

React ne définit pas lui-même d'encapsulation CSS.

Il donne surtout `className` et `style`, puis laisse le choix à CSS, CSS Modules, CSS-in-JS ou au framework.

Documentation : https://react.dev/reference/react-dom/components/common

## Vue

Vue propose `<style scoped>`.

Le compilateur ajoute un attribut unique et réécrit les sélecteurs.

Cela évite que le style d'un composant déborde vers les autres.

Documentation : https://vuejs.org/api/sfc-css-features

## Svelte

Svelte scope son CSS de composant avec une classe générée à partir d'un hash.

Documentation : https://svelte.dev/ et la documentation du compilateur.

## Angular

Angular utilise par défaut une encapsulation émulée.

Il génère un attribut unique dans le HTML et dans les sélecteurs.

Documentation : https://angular.dev/guide/components/styling

## Web Components

Le Shadow DOM isole fortement.

Pour laisser le parent restyler une partie choisie, le standard offre :

- `part`
- `::part()`
- `exportparts`

Documentation : https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Shadow_parts

## HoloCode actuel

Le moteur produit :

- une classe racine de composant : `holo-c-ArticleCard` ;
- les noms de style : `holo-s-promo` ;
- des variables CSS normales.

C'est une très bonne base.

---

# 7. Le défaut que je corrigerais : la fuite des styles

## Vérifié dans le code

`plat.rs` transforme :

```holo
.card { color: red; }
```

en gros en :

```css
.holo-s-card { color: red; }
```

La règle reste globale à la page.

Un élément extérieur au composant avec le même nom `.card` reçoit donc le même style.

De plus, `holo.rs` fusionne les styles importés par cible et conserve un premier style importé lorsqu'un second import vise la même cible.

Deux bibliothèques utilisant `.card` peuvent donc se gêner.

## C'est plus important que `slot`

Un système de composants réutilisables doit pouvoir être importé sans polluer les autres composants.

Sinon on recrée exactement les collisions CSS que les composants étaient censés réduire.

## Ce que je recommande

**Ne pas demander au débutant d'écrire un sélecteur composé. Le compilateur doit scoper pour lui.**

### Pour un composant importé

Un fichier :

```holo
Part(name: ArticleCard, ...)
.card { padding: 8px; }
```

peut être compilé vers quelque chose comme :

```css
.holo-c-ArticleCard .holo-s-card,
.holo-c-ArticleCard.holo-s-card {
  padding: 8px;
}
```

ou vers un attribut/classe de scope généré.

L'auteur continue d'écrire seulement :

```holo
.card { ... }
```

### Pourquoi je ne prendrais pas `@scope` comme seule solution

`@scope` est maintenant Baseline 2026, mais reste nouveau et peut manquer sur des téléphones/navigateurs anciens.

Source : https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/%40scope

HoloCode veut précisément garder une page plate sur du matériel ancien.

Je préfère donc un **scoping généré à la Vue/Svelte/Angular**, qui marche avec du CSS classique.

### Variables CSS

Je garderais les variables.

Elles sont parfaites pour les « ouvertures » volontaires du composant :

```holo
ArticleCard { --accent: #E9B44C; }
.promo { --accent: crimson; }
```

C'est simple et natif.

### Variantes déclarées

Pas nécessaires maintenant.

`ArticleCard.promo(...)` suffit.

Introduire une liste `variants:` ajouterait un concept pour contrôler quelque chose que le système sait déjà faire.

### Parties nommées de l'extérieur

Pas maintenant.

Un équivalent de `::part()` devient utile quand un composant doit exposer précisément :

- son titre ;
- son image ;
- son bouton.

Mais cela oblige à créer un contrat public d'internals.

À ajouter seulement quand un vrai composant en a besoin.

---

# 8. Un emplacement `children` / slot

## Est-ce utile ?

Oui.

Exemples :

- `Dialog` réutilisable avec contenu libre ;
- `Card` dont le contenu change complètement ;
- `Panel` ;
- `Layout`.

## Est-ce indispensable avant la 3D ?

**Non.**

Pour le site d'artisan servant de seuil « assez utilisable », les paramètres + `Repeat` suffisent.

Je placerais le slot **après les listes à champs**, sauf si un vrai exemple bloque avant.

C'est cohérent avec la règle du projet : ne pas ajouter de mot sans besoin réel.

---

# 9. Pièges des composants

## Récursion directe

Déjà refusée.

Bon choix.

## Récursion indirecte

Le code garde un chemin des composants et refuse aussi le cycle indirect.

Bon choix.

## Profondeur

8 niveaux au plus.

Suffisant pour V1.

## Copies

2 000 au plus.

C'est un garde-fou raisonnable, pas une promesse de performance.

## Collision de noms de composants importés

Déjà refusée :

> deux composants s'appellent « X »

Bon choix.

## Collision de styles importés

Pas suffisamment isolée aujourd'hui.

À corriger.

## HTML

Le composant est déplié en vrai HTML.

Excellent choix.

Il n'ajoute pas un Shadow DOM ni un custom element inutile.

## Accessibilité

Le composant hérite de l'accessibilité des blocs qu'il contient.

Par exemple :

- `Button` devient un vrai `<button>` ;
- `Image` exige `alt` ;
- `Input` produit un vrai `<label>` ;
- `Main`, `Header`, `Footer` produisent la structure sémantique.

C'est plus sûr qu'un composant qui serait dessiné dans un canvas.

## Poids

Comme le composant est déplié, il ne demande pas un runtime de composants distinct.

Bon choix.

Le coût principal reste le moteur interactif lui-même.

---

# 10. Les sondes écrites pour cette revue

Le fichier `sondes.rs` de cette proposition couvre notamment :

1. un paramètre remplacé correctement ;
2. un bouton renommé par copie ;
3. une règle du composant qui agit sur la page ;
4. un composant dans `Repeat` ;
5. un paramètre qui collisionne avec `State` ;
6. la récursion ;
7. deux composants importés du même nom ;
8. le HTML sémantique produit ;
9. les classes de variante ;
10. la fuite actuelle d'un nom de style importé.

Ces sondes complètent les tests déjà présents dans `composants.rs`.

Je ne les marque pas « passées » : je n'ai pas pu relancer Cargo dans cette session.

---

# 11. Le tableau de Claude, rempli à nouveau

Ces notes restent un jugement.

Je garde **exactement les mêmes critères et les mêmes poids**.

| Critère | Poids | TypeScript | React | Angular | Vue | Svelte | SolidJS | Astro | Elm | Flutter | **HoloCode actuel** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Facile pour un débutant | 20 | 35 | 45 | 30 | 70 | 80 | 55 | 75 | 45 | 65 | **90** |
| Court à écrire | 10 | 35 | 65 | 45 | 80 | 90 | 75 | 80 | 45 | 50 | **88** |
| Erreurs attrapées tôt, bien expliquées | 10 | 85 | 75 | 90 | 75 | 80 | 78 | 75 | 100 | 92 | **92** |
| Les valeurs qui changent | 10 | 25 | 70 | 85 | 90 | 90 | 97 | 45 | 90 | 80 | **75** |
| Les composants réutilisables | 10 | 30 | 90 | 90 | 90 | 88 | 82 | 82 | 70 | 95 | **78** |
| Accessibilité et vrai HTML par défaut | 10 | 50 | 55 | 70 | 70 | 82 | 60 | 85 | 65 | 45 | **82** |
| Rapidité et poids | 10 | 95 | 60 | 55 | 75 | 92 | 95 | 100 | 85 | 40 | **68** |
| Ce qu'on peut construire | 10 | 100 | 100 | 100 | 95 | 92 | 88 | 75 | 65 | 95 | **60** |
| Les outils | 5 | 95 | 95 | 90 | 85 | 85 | 70 | 82 | 65 | 95 | **40** |
| Les bibliothèques, l'entraide | 5 | 100 | 100 | 85 | 85 | 75 | 50 | 70 | 30 | 85 | **5** |
| **Total pondéré** | **100** | **58,8** | **70,3** | **68,3** | **80,0** | **85,4** | **74,5** | **76,8** | **65,8** | **71,7** | **74,6** |

## Pourquoi HoloCode monte par rapport au tableau de Claude

Le tableau de Claude donnait 45 aux composants.

Ce n'est plus le code actuel.

Depuis :

- `params` existe ;
- appel comme bloc ordinaire ;
- règles par copie ;
- composant dans `Repeat` ;
- variables CSS ;
- plusieurs noms de style ;
- protection contre récursion et collision de noms.

Je mets donc **78**, pas 45.

Je ne mets pas 90 car :

- pas de slot ;
- pas d'état local ;
- styles internes pas encore réellement isolés par scope ;
- données structurées manquantes.

## Pourquoi je baisse « facile » et « court »

95 me paraît trop généreux.

HoloCode est très lisible, mais le GUIDE contient maintenant beaucoup de notions.

Le projet lui-même parle d'environ 290 mots.

Et la liste de tâches n'est pas réellement la plus courte des dix.

90 / 88 restent excellents.

## Accessibilité : 80 était-elle absurde ?

Non.

Le code est meilleur que beaucoup de frameworks par défaut sur certains points :

- vrai HTML ;
- `alt` obligatoire ;
- labels natifs ;
- boutons natifs ;
- landmarks ;
- tableaux sémantiques ;
- pas de canvas pour la page plate.

Mais aucune revue TalkBack/NVDA complète n'est encore consignée.

Je mets **82**, pas 95.

L'essai réel doit décider si cette note monte.

---

# 12. Vérification des nombres de lignes

J'ai recompté les blocs de code réellement présents dans le fichier.

Je compte ici les lignes non vides dans chaque bloc affiché.

| Exemple | Claude | Recompte du bloc fourni |
|---|---:|---:|
| HoloCode | 22 | **24** |
| TypeScript seul | 24 de code + 9 HTML | **24** lignes TS non vides ; le HTML annoncé n'est pas montré |
| React | ~22 | **21** |
| Angular | ~27 | **27** |
| Vue | ~18 | **17** |
| Svelte | ~15 | **15** |
| SolidJS | ~21 | **21** |
| Flutter | ~38 | **34** non vides, 38 physiques |

Les ordres de grandeur de Claude sont donc globalement honnêtes.

Deux réserves :

1. le chiffre TypeScript « 33 » dépend d'un HTML annoncé mais absent du bloc ;
2. comparer des lignes dépend énormément du formatage.

Je n'utiliserais pas les lignes comme métrique principale.

---

# 13. Poids : la page qui bouge fait-elle vraiment ~560 Ko ?

## Vérifié dans le dépôt

ADR-033 parle encore d'environ 560 Ko au moment de la décision.

Le README mesure sur un commit antérieur :

- WASM Brotli : **502 435 octets** ;
- JavaScript associé alors : **14 373 octets**.

Donc environ **517 Ko transférés** pour ce build mesuré, hors page.

Le fichier actuel `moteur/web/page-moteur.js` fait maintenant **73 057 octets non compressés** dans GitHub.

Le WASM actuel n'est pas versionné dans le dépôt sous `web/pkg`, donc je ne peux pas donner son poids 2026-10-07 sans reconstruction.

## Conclusion

Dire « environ 560 Ko » reste un **ordre de grandeur historique raisonnable**, mais ce n'est pas une mesure du commit actuel.

Je remplacerais dans un futur comparatif :

> le moteur pèse environ 560 Ko

par :

> le dernier build consigné était autour de 0,5 Mo Brotli ; le build courant doit être remesuré.

---

# 14. Opinion publique : correction avec les sources officielles

## Stack Overflow 2026

La page officielle est disponible maintenant :

https://survey.stackoverflow.co/2026/technology/data/web-framework

Usage 2026, tous répondants :

- React : **41,5 %**
- Vue.js : **17,2 %**
- Angular : **16,0 %**
- Svelte : **7,8 %**
- Astro : **6,2 %**
- SolidJS : **1,5 %**
- htmx : **5,4 %**

Admired 2026 :

- React : **46,7 %**
- Vue.js : **47,2 %**
- Angular : **42,1 %**
- Svelte : **59,4 %**
- SvelteKit : **64,4 %**
- Astro : **58,8 %**
- SolidJS : **55,5 %**
- htmx : **55,9 %**

Ces chiffres sont plus fiables que les extraits repris dans le comparatif.

Ils montrent notamment que :

- Svelte reste très apprécié ;
- Astro reste très apprécié ;
- SolidJS est beaucoup plus petit et, dans cette enquête, pas à 90 % d'admiration ;
- htmx est admiré, mais pas à 73 % dans l'édition 2026.

## Stack Overflow 2025

La page officielle confirme que Phoenix était le framework web le plus admiré en 2025, à environ 79 %.

Source :

https://survey.stackoverflow.co/2025/technology

Le tableau de Claude avait donc tort de dire que Svelte était « 1er des frameworks » dans Stack Overflow 2025.

Svelte pouvait être premier dans un sous-ensemble ou un extrait, mais pas dans le classement global officiel des frameworks web.

## State of JS 2025

Pages officielles consultées :

- front-end : https://2025.stateofjs.com/en-US/libraries/front-end-frameworks/
- meta-frameworks : https://2025.stateofjs.com/en-US/libraries/meta-frameworks/

La page officielle confirme notamment :

- Astro en tête de satisfaction des méta-frameworks ;
- la forte polarisation / baisse de satisfaction de Next.js ;
- Solid et Svelte très bien placés parmi les frameworks front-end.

Je n'ai pas pu extraire proprement tous les pourcentages exacts des graphiques interactifs via l'outil de recherche.

Je ne reprendrais donc pas les valeurs « 90 % », « 94 % », etc. comme des chiffres vérifiés sans export officiel du graphique.

---

# 15. Point 2 de Claude : listes à champs

## C'est maintenant le chantier web numéro 1

Aujourd'hui deux systèmes coexistent :

### Liste statique riche

```holo
Repeat(
  items: [
    Item(key: sunrise, title: "Sunrise", price: 120),
  ],
)
```

Elle a des champs.

### Liste dynamique de `State`

```holo
State(tasks: ["Pain", "Lait"])
```

Elle ne contient que des textes.

Le serveur `Data` ne remplit que des scalaires.

Voilà le trou.

## Ce qu'il faut

Une liste dynamique d'éléments bornés avec champs.

Exemple conceptuel :

```holo
State(
  products: [
    Item(key: sunrise, title: "Sunrise", price: 12000),
  ],
)
```

ou une liste alimentée par :

```holo
Data(from: "products.json")
```

Puis :

```holo
Repeat(
  over: products,
  children: [
    ArticleCard(
      title: item.title,
      price: item.price,
    ),
  ],
)
```

## Garde-fous nécessaires

- nombre maximal d'éléments ;
- nombre maximal de champs ;
- texte maximal ;
- types bornés ;
- clés stables ;
- taille JSON maximale ;
- aucun objet récursif ;
- aucun champ arbitraire transformé en mot du langage.

C'est le changement qui débloque réellement :

- catalogues ;
- messages ;
- résultats de recherche ;
- commandes ;
- profils ;
- listes venant d'un serveur.

---

# 16. Point 3 : disposition

Claude mélangeait plusieurs choses, dont certaines sont déjà faites.

## Déjà fait depuis son tableau

- plusieurs noms de style par bloc : **fait** ;
- `width`, `height`, `max-width` existent dans les styles ;
- `Row`, `Column`, `Grid`, `Stack` existent.

## Ce qui manque réellement

### Prendre la place restante

Il manque l'équivalent d'un `Expanded` Flutter / `flex-grow` web.

Je proposerais d'abord un réglage de bloc, pas du CSS libre de disposition.

Par exemple, seulement si un cas réel le justifie :

```holo
Row(children: [
  Input(...),
  Button(...),
])
```

avec un mécanisme très court sur l'enfant qui doit grandir.

Le nom est à discuter ; je ne le fixe pas ici.

### Fichier de styles séparé

Utile, mais pas prioritaire.

Le système `import` sert déjà au thème partagé.

Avant d'ajouter un deuxième mécanisme de fichiers, je vérifierais si :

```holo
import "theme.holo"
```

avec un `Part` vide ou dédié au thème suffit vraiment mal.

### Isolation CSS des composants

C'est, pour moi, plus urgent qu'un fichier de styles séparé.

---

# 17. Point 4 : moteur interactif allégé

## Le but est bon

Une page lue n'envoie déjà presque rien grâce à ADR-033.

Mais au premier bouton, elle charge tout le moteur, y compris le code de rendu profond.

## Le découpage actuel

Dans `Cargo.toml`, `wgpu`, `web-sys`, `wasm-bindgen` sont rattachés au build WASM.

`lib.rs` compile aussi :

- `rendu`
- `web`

Le même artefact contient donc :

- parseur HoloCode ;
- arbitre ;
- HTML dynamique ;
- règles ;
- listes ;
- composants ;
- navigation profonde ;
- mosaïque ;
- GPU.

## Peut-on viser moins de 100 Ko ?

**Possible comme objectif d'expérience, pas comme promesse.**

Il faut produire un **second artefact WASM**, pas essayer de « désactiver » wgpu après téléchargement.

### Découpe que je testerais

Artefact léger :

- `holo`
- `blocs`
- `composants`
- `etat`
- `listes`
- `repetition`
- `regles`
- `format`
- petite partie de `plat` nécessaire aux mises à jour
- API formulaires/données/modules selon besoin

Sans :

- `wgpu`
- `rendu`
- `mosaique`
- `navigation`
- `univers`
- vue points

Artefact profond :

- le reste.

### Cible

Je viserais :

- **< 150 Ko Brotli** d'abord ;
- puis tenter **< 100 Ko**.

Pourquoi 150 avant 100 ?

Parce qu'un objectif arbitraire ne doit pas pousser à une architecture tordue.

Mesurer d'abord la taille minimale honnête.

---

# 18. Point 5 : outils

Je suis d'accord avec Claude, mais je changerais l'ordre interne.

## 1. Voir les valeurs

Le plus utile.

Un panneau de développement qui montre :

- état numérique ;
- textes ;
- listes ;
- dernier signal ;
- dernière règle appliquée ;
- dernier refus.

Sans cela, un non-programmeur ne comprend pas pourquoi « ça ne bouge pas ».

## 2. Mise en forme automatique

Très utile parce que la syntaxe devient plus riche.

Le formateur doit produire **une seule écriture officielle**.

## 3. Essais écrits

Très important, mais légèrement après le débogueur.

Conceptuellement :

```
quand Add est touché
alors cart vaut 1
```

La forme exacte est à décider plus tard.

Le moteur possède déjà un arbitre déterministe : cela rend ces essais particulièrement naturels.

---

# 19. Point 6 : lecteur d'écran

## À faire avant la 3D

Oui.

Pas parce que le code semble mauvais.

Justement parce qu'il semble assez bon pour mériter une vraie vérification.

## Minimum

### TalkBack Android

Tester :

- navigation par titre ;
- landmarks ;
- formulaire ;
- erreurs ;
- Dialog ;
- menu ;
- bouton qui attend le moteur ;
- changement de panier ;
- Repeat dynamique.

### NVDA Windows

Même scénario.

## Ce que le test doit vérifier

- ordre annoncé ;
- noms accessibles ;
- focus après mise à jour ;
- annonce ou non des changements d'état ;
- `aria-busy` pendant le chargement du moteur ;
- Dialog correctement annoncé ;
- erreurs de formulaire compréhensibles.

Le vrai risque n'est pas le HTML initial.

Le risque est **ce qui change après l'arrivée du moteur**.

---

# 20. Ce qui manque au plan de Claude

## A. Isolation CSS des composants

À ajouter.

C'est mon ajout principal.

## B. Identités stables pour les futures listes à champs

À décider avant de rendre les listes dynamiques.

Une ligne ne doit pas être identifiée seulement par son rang.

Sinon supprimer/reclasser un élément peut faire pointer une interaction vers le mauvais objet.

## C. Budget de données réseau

`Data` doit avoir un plafond de taille.

Une liste structurée venant du serveur peut grossir beaucoup plus vite que quelques scalaires.

## D. Chargement et erreur de composant importé

Les messages sont déjà corrects, mais l'éditeur devrait pouvoir ouvrir le fichier importé à la bonne ligne.

Ce n'est pas bloquant pour V1.

## E. Sécurité des contenus texte riches

Les données serveur et champs de listes doivent rester échappés.

Ne jamais permettre qu'un champ reçu devienne une balise HoloCode ou du HTML.

Le code actuel le fait déjà correctement pour les listes de textes.

Il faudra conserver cette propriété.

---

# 21. Ce qui est inutile avant la 3D

Je repousserais :

- état local de composant ;
- système de variantes déclaré ;
- équivalent de `::part()` ;
- slots multiples nommés ;
- CSS-in-JS ;
- Shadow DOM ;
- fichier CSS externe dédié ;
- bibliothèque de composants distante ;
- hydratation par composant ;
- serveur de composants.

Tout cela peut venir un jour.

Rien de cela n'est nécessaire pour que le web soit « assez utilisable ».

---

# 22. Ordre recommandé

Du plus important au moins important :

1. **Corriger l'isolation CSS des composants importés.**
2. **Ajouter les listes dynamiques à champs avec clés stables.**
3. **Permettre à `Data` de remplir ces listes, avec plafonds stricts.**
4. **Mesurer et construire un artefact WASM interactif sans le rendu 3D.**
5. **Ajouter la petite capacité de disposition qui prend la place restante, seulement avec un vrai exemple.**
6. **Ajouter un panneau qui montre valeurs, listes, dernier signal et dernière règle.**
7. **Ajouter le formateur officiel.**
8. **Ajouter les essais écrits de l'arbitre.**
9. **Faire TalkBack + NVDA et corriger ce qu'ils trouvent.**
10. **Faire l'essai avec cinq débutants.**
11. **Seulement si un exemple réel bloque : ajouter un slot de composant.**
12. **Ensuite commencer la 3D.**

---

# Verdict final

**La version web n'a plus besoin d'une refonte des composants : leur base est bonne ; avant la 3D, je corrigerais surtout leur isolation CSS, puis les données structurées dynamiques et le poids du moteur interactif, car ce sont maintenant les trois obstacles réels à un HoloCode web “assez utilisable”.**
