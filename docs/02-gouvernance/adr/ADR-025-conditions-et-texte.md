# ADR-025 — Les conditions (`If`), et le texte qui manquait (`Hr`, `Quote`, `Code`, `alt`)

- Statut : ACCEPTÉ
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; `docs/01-holocode/COMPARATIF-CONCURRENTS.md` (planning, étape 1)
- Validation : Yocthan, le 2026-10-04, sur le planning en sept étapes : « Oui, commence. » L'écriture est une proposition de Claude ; à juger après essai. Validé par Yocthan le 2026-10-06 : « Qu'est-ce que tu attends pour valider tous ceux qui sont à l'essai ? »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Étape 1 du planning : faire passer HoloCode de 35 % à 50 % de ce que permet le meilleur outil du web. Une page savait retenir des valeurs (`ADR-023`), mais pas s'adapter à elles : le bouton « Vider le panier » s'affichait même devant un panier vide. Et il manquait des choses simples : un trait, une citation, du texte montré tel quel, un retour à la ligne, le texte qui remplace une image.

## Décision (à l'essai)

### Les conditions

```holo
Page(
  title: "My shop",
  state: State(cart: 0),
  children: [
    Button(name: Add, text: "Add"),
    If(cart, is: 0, children: [
      "Your cart is empty.",
    ]),
    If(cart, over: 0, under: 10, children: [
      Button(name: Empty, text: "Empty the cart"),
    ]),
  ],
  rules: [
    On(Add.tap, effect: cart.add(1)),
    On(Empty.tap, effect: cart.set(0)),
  ],
)
```

1. **`If(valeur, comparaison, children: [...])`** : ce qu'il contient ne se montre que si la condition est vraie. Quand la valeur change, la page suit toute seule.
2. **Quatre comparaisons, en mots** : `is` (égal), `not` (différent), `over` (plus grand), `under` (plus petit). On compare à un nombre entier.
3. **Plusieurs comparaisons valent ensemble** : `over: 0, under: 10` veut dire « entre 1 et 9 ».
4. La valeur est une valeur de `State`, ou une valeur calculée par le moteur (`count`, `total`).
5. Il n'y a pas de « sinon » : on écrit une seconde condition. Il n'y a pas de « ou ».

### Le texte

6. **`Hr()`** : un trait de séparation.
7. **`Quote("…", by: "…")`** : une citation, avec son auteur si on le donne.
8. **`Code("…")`** : du texte montré tel quel, lettre pour lettre. Dans une phrase, on l'entoure d'accents graves.
9. **Le retour à la ligne** : un texte écrit entre trois guillemets garde ses retours à la ligne.
10. **`Image(alt: "…")`** : ce que montre l'image, pour qui ne la voit pas. Sans `alt`, l'image est tenue pour un décor.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| La forme d'une condition | un bloc `If(...)` ; un réglage `visible:` sur chaque bloc ; une règle `On(...)` qui montre et cache | Un bloc. On voit dans le plan de la page ce qui dépend de quoi, et une condition peut contenir plusieurs blocs. |
| Le mot | `If` ; `When` ; `Show` | `If`. Tout le monde le connaît, programmeur ou non. `When` se confondrait avec `On`, qui parle d'un moment. |
| Les comparaisons | des signes (`cart > 0`) ; des mots (`over: 0`) | Des mots. Un signe demande une expression, et une expression est le premier pas vers le code libre (`ADR-015`). Les mots sont des réglages comme les autres. |
| Les mots des comparaisons | `above`/`below` ; `over`/`under` ; `more`/`less` | `over` et `under`. `above` est déjà pris (un point planté « au-dessus » d'un bloc). |
| Le « sinon » | `else:` ; une seconde condition | Une seconde condition. `else` suppose qu'on lise la première pour comprendre la seconde ; deux conditions écrites en clair se lisent seules. |
| Le trait, la citation | `Hr`, `Quote` (mots de HTML, ou proches) ; `Line`, `Citation` | `Hr`, comme `P` et `A` ; `Quote`, plus court que `blockquote` et sans le défaut de HTML qui en a trois (`blockquote`, `q`, `cite`). |
| `alt` : obligatoire ou non | obligatoire ; facultatif | Facultatif pour l'instant, à regret. L'obliger casserait les cas de la suite de conformité, que les prototypes des autres IA lisent aussi. À revoir avec eux. |

Défauts du web évités : en JavaScript, montrer et cacher se fait à la main, à chaque changement, et l'oubli fait mentir l'écran ; en HTML, rien ne dit qu'un bloc dépend d'une valeur. En Markdown et en HTML, un retour à la ligne dans le texte est ignoré : ici, il est gardé.

Défaut du web **non** évité : `alt` peut encore être oublié, comme en HTML.

## Conséquences

### Positives

- Une page s'adapte à ce qui se passe : panier vide, livraison offerte au-delà d'un montant.
- Premier pas vers le jeu : « si le score atteint 10 » s'écrira de la même façon.

### Négatives et risques

- On ne compare qu'à un nombre écrit dans le fichier, pas à une autre valeur.
- Pas de « ou ».
- **Les listes répétées, annoncées dans l'étape 1, ne sont pas faites.** Répéter un bloc pour chaque article demande des valeurs qui soient des listes ; nous n'avons que des nombres. C'est reporté à l'étape où les valeurs s'enrichissent.

**Corrigé le même jour.** Les conditions étaient d'abord calculées à deux endroits : en Rust au premier affichage, en JavaScript ensuite. Elles le sont maintenant à un seul, dans le moteur (`etat::conditions`) ; la page d'entrée ne compare plus rien, elle cache ce que le moteur dit faux.

## Ce qui reste à faire

- Une condition dans une règle : `On(Add.tap, if: …, effect: …)`, pour le jeu.
- Comparer deux valeurs entre elles.
- Rendre `alt` obligatoire, avec l'accord des autres IA.

## Critères de validation

- `exemples/boutique-comparee/boutique.holo` : « Your cart is empty » disparaît au premier ajout ; le code de livraison apparaît à partir de 300 euros.
- Tests du moteur : `etat.rs` (`une_condition_montre_ou_cache_selon_une_valeur`), `plat.rs` (`le_texte_qui_manquait_…`).

## Conditions de réexamen

- Quand Yocthan aura essayé et jugé : `If`, `is`, `not`, `over`, `under`, `Hr`, `Quote`, `by`, `Code`, `alt`.
- Au premier jeu, qui dira si ces conditions suffisent.
