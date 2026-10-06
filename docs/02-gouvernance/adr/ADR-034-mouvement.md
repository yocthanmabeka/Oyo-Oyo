# ADR-034 — Le mouvement : `Enter`, `Loop`, `Scenes`

- Statut : ACCEPTÉ
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 (« fais du motion design en HoloCode et en HTML, CSS, JavaScript, tous les curseurs à 100 % » ; « fais une amélioration, sinon il sera battu »)
- Validation : Yocthan, le 2026-10-04, pour un essai. L'écriture est une proposition de Claude ; les noms restent à revoir avec les autres. Validé par Yocthan le 2026-10-06 : « Qu'est-ce que tu attends pour valider tous ceux qui sont à l'essai ? »
- Projets affectés : HoloCode, HoloEngine

## Contexte

HoloCode savait déplacer une forme sur un plateau, pas à pas, par une horloge. Il ne savait ni choisir le caractère d'un mouvement, ni tourner, grandir, flouter, ni faire arriver un texte lettre par lettre, ni enchaîner des scènes. Yocthan voulait comparer un film de motion design écrit en HoloCode à son jumeau en HTML, CSS et JavaScript, après avoir amélioré HoloCode.

## Décision

```holo
Page(
  title: "Motion",
  children: [
    H1("Hello", enter: Enter(y: 40px, opacity: 0, letters: 0.05s, ease: spring)),
    Shape(form: circle, color: "#E9B44C", size: 60px,
      enter: Enter(scale: 0, at: 0.6s, ease: back),
      loop: Loop(scale: 1.2, for: 0.8s)),
    Scenes(height: 200px, repeat: forever, children: [
      Scene(for: 2s, children: [ H2("One", enter: Enter(x: -200px, opacity: 0)) ]),
      Scene(for: 2s, children: [ H2("Two", enter: Enter(scale: 3, opacity: 0, blur: 10px)) ]),
    ]),
  ],
)
```

1. **`enter: Enter(…)`**, sur tout bloc qui se voit : on écrit **d'où il part** ; il arrive à sa place.
2. **`loop: Loop(…)`** : on écrit **où il va** ; il y va et revient, sans fin. `back: false` : il recommence sans revenir.
3. Dix choses bougent : `opacity`, `x`, `y`, `scale`, `rotate`, `flip`, `tilt`, `blur`, `hue`, `round`. Toutes bornées, avec leur unité.
4. `at:`, `for:`, `ease:` ; sept caractères nommés : `linear`, `smooth`, `out`, `in`, `back`, `spring`, `bounce`.
5. `letters:` (lettre après lettre, sur un texte) et `each:` (enfant après enfant, sur un bloc qui a des enfants).
6. **`Scenes` et `Scene(for:)`** : des scènes qui passent l'une après l'autre au même endroit ; `repeat: forever`. Dans une scène, `at:` compte depuis son début.
7. Le moteur fabrique du CSS. La page bouge sans attendre le moteur (`ADR-033`) et ne pèse rien de plus. Qui demande moins de mouvement voit la page arrêtée (pour des scènes, la dernière).
8. Rien ne bouge si l'auteur ne l'écrit pas, comme la 3D.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Comment décrire une entrée | des images clés (de… vers…), comme CSS ; un mot tout fait (`fade`, `slide`) ; **seulement le départ** | Le départ. Le repos est déjà écrit : c'est le bloc lui-même. On écrit deux fois moins qu'avec des images clés, et plus librement qu'avec des mots tout faits. |
| Où l'écrire | un style ; un bloc à part qui vise le bloc par son nom ; **un paramètre du bloc** | Un paramètre. Le mouvement se lit à côté de ce qui bouge. CSS sépare les deux et oblige à inventer un nom d'animation. |
| Les courbes | des nombres (`cubic-bezier(…)`) ; **des noms** | Des noms. Personne ne lit `cubic-bezier(.34,1.56,.64,1)` ; tout le monde comprend `back`. Le ressort et le rebond, impossibles en `cubic-bezier`, sont offerts. |
| Enchaîner | calculer les délais à la main ; un chef d'orchestre en JavaScript ; **des scènes** | Des scènes. Chaque scène dit sa durée ; le moteur calcule le reste, y compris la boucle sans fin. |
| Lettre à lettre | du JavaScript qui coupe le texte ; **`letters:`** | `letters:`. Le moteur coupe le texte et garde les mots entiers. |

Défauts du web évités : des noms d'animation à inventer et à relier ; des délais recalculés à la main à chaque changement ; des courbes en chiffres ; le texte coupé en `span` par du JavaScript ; des animations qui ignorent la préférence « moins de mouvement ».

## Le duel

Le même film en sept scènes, deux fois : `exemples/motion/holocode/showreel.holo` et `exemples/motion/web/showreel.html`. Résultats dans `exemples/motion/README.md`.

## Conséquences

### Positives

- Un film de motion design s'écrit sans programmer.
- Il ne demande pas le moteur dans le navigateur : 10 Ko téléchargés pour le film HoloCode.

### Négatives et risques

- Pas de particules par centaines, pas de chemin dessiné à suivre, pas de forme qui se change en une autre forme quelconque (seulement un carré qui s'arrondit), pas de mouvement qui suit la souris : le film en JavaScript les a, HoloCode non.
- `hue` tourne les couleurs, il ne va pas vers une couleur choisie.
- Les tailles de texte sont fixes (en px) : un grand titre peut déborder d'un petit écran.
- Sans listes répétées, 36 éclats s'écrivent un par un.
- Pas mesuré sur un téléphone bon marché.

## Critères de validation

- Les leçons 32, 33, 34 ; le film `exemples/motion/holocode/showreel.holo`.
- Tests du moteur : `plat.rs` (`le_mouvement_devient_du_css_et_ne_bouge_que_si_on_l_ecrit`), `mouvement.rs`.

## Conditions de réexamen

- Quand Yocthan aura vu les deux films.
- Au premier besoin de particules, de chemins ou de mouvement qui répond au doigt.
