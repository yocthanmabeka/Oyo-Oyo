# ADR-017 — La forme : un thème, des styles nommés à point, des réglages par bloc ; les couleurs d'un point

- Statut : ACCEPTÉ pour le principe ; la place des styles dans le fichier reste à décider
- Date : 2026-10-03
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, revue Codex du 2026-10-03 (`proposals/GPT5.6/revue-2026-10-03/`), journal du 2026-10-03
- Validation : décidé par Yocthan le 2026-10-03, en discussion avec Claude.
- Projets affectés : HoloCode, HoloCompiler, HoloEngine

## Contexte

Le CSS réglait la forme de l'ancien web. Yocthan l'appréciait pour une raison précise : on définit un style une fois, on lui donne un nom avec un point (`.card`), et on l'applique partout. Par ailleurs, les couleurs du Big Bang ne sont écrites nulle part : le moteur les tire de la graine.

## Décision

**Quatre niveaux pour la forme**, du plus général au plus précis ; le plus précis l'emporte :

1. le thème : le style de la `Page` ou du `World`, que leur contenu reprend ;
2. le style d'un type de bloc (`P { … }`) ;
3. un style nommé, avec le point du CSS (`.card { … }`), posé sur un bloc par `P.card(...)` ;
4. un paramètre écrit sur le bloc corrige un seul élément.

Ni sélecteurs compliqués, ni cascade cachée.

### L'écriture : celle du CSS de base (décidé le 2026-10-03)

Yocthan a tranché la question laissée ouverte : **le style s'écrit comme le CSS que tout le monde connaît**, avec des accolades, et non sous la forme d'un bloc `Style(...)` répété. Sa raison : écrire un style en CSS est simple, « trop facile même » ; une écriture « comme dans tous les langages » ramènerait la complexité qu'on veut éviter. Claude recommandait une écriture en blocs ; Yocthan a décidé autrement.

```holo
Page(
  title: "My shop",
  children: [
    H1("My shop"),
    P.card("Free delivery from 30 euros."),
    Button.card(name: Open, text: "Enter the workshop"),
  ],
)

Page { background: #101020; color: white; }
H1 { color: #E9B44C; font-size: 32px; }
.card {
  background: #1a1a2e;
  border-radius: 12px;
  padding: 8px 16px;
}
```

Les styles viennent après le bloc racine. Les blocs `Theme(...)` et `Style(...)` de la première version de cette fiche disparaissent : garder les deux écritures aurait fait deux façons de dire la même chose.

### Sept règles pour que le style serve aussi aux jeux (validées par Yocthan le 2026-10-03)

Yocthan a relevé que le style d'un jeu n'est pas celui d'un site : un jeu parle de matériaux et de thèmes, qu'on pose sur des objets. Pour ne pas tout refaire en arrivant aux jeux, on garde du CSS son écriture, pas son mécanisme.

1. **Un seul mot, le style, pour le web et pour le jeu** : un paquet nommé de réglages d'apparence. Les réglages de profondeur (lumière émise, texture) s'ajouteront aux mêmes styles ; il n'y a pas de bloc « matériau » à part.
2. **Deux façons seulement de viser** : par type de bloc (`P`) et par nom (`.card`). Pas de sélecteur composé. Un bloc porte un seul nom de style.
3. **Un style ne dit que l'apparence, jamais la disposition.** `display`, `position`, `float` et leurs semblables sont refusés ; la disposition vient des blocs dans une page, de la position dans un monde.
4. **Un thème par monde** : le style de `World` règle l'ambiance d'un monde, celui de `Page` celle d'une page.
5. **Un fichier de styles s'importe comme une ressource**, ou les styles s'écrivent à la fin du fichier pour une petite page.
6. **Chaque réglage est vérifié** : un réglage inconnu, une valeur mal écrite, un `;` oublié, un style défini deux fois ou jamais défini sont refusés avec leur ligne.
7. **L'apparence ne change en cours de route que par une règle du monde** (`On(...)`), jamais par du code qui modifie un style de n'importe où.

### Défauts de HTML, CSS et JavaScript évités

- CSS : un réglage mal orthographié ignoré en silence ; un `;` oublié qui avale la ligne suivante ; la cascade et la spécificité, où l'ordre des règles change le résultat ; la disposition mêlée à l'apparence ; plusieurs noms pour le même réglage (`background` et `background-color`).
- HTML : une classe inconnue qui ne fait rien et ne dit rien.
- JavaScript : `element.style` modifiable de partout, sans trace.

**Trois possibilités pour la couleur d'un point :**

```holo
Point(name: A, seed: 42)                                    // la graine décide
Point(name: B, seed: 42, color: "#E9B44C")                  // l'auteur impose la couleur de ce point
Point(name: C, seed: 42, palette: ["#E9B44C", "#245C45"])   // l'auteur impose les couleurs de ses enfants
```

## Ce qui reste à faire

- Importer un fichier de styles : les imports ne sont pas encore pris en charge par le moteur.
- Les réglages propres à la profondeur (lumière émise, texture) et les animations.
- Les réglages de couleur écrits directement sur un bloc (niveau 4) ne sont pas encore vérifiés.
- Les accolades restent interdites à l'intérieur d'un bloc (`ADR-015`) : elles n'existent que pour les styles, qui ne contiennent que des réglages.

## Conséquences

### Positives

- La priorité se lit d'un coup d'œil ; un thème évite vingt corrections répétées.

### Négatives et risques

- Les styles sont lus et vérifiés par le moteur (`moteur/src/styles.rs`), mais pas encore appliqués à l'écran.
- Le vérificateur est plus strict que le CSS : seuls quinze réglages de base sont connus, les tailles sont en `px` ou en `%`, et une trentaine de noms de couleur sont acceptés en plus de l'écriture `#E9B44C`. La liste grandira avec les besoins.
- Une couleur tirée d'une graine peut donner un texte illisible : les textes et les boutons prendront leurs couleurs du thème, pas du hasard (remarque de Codex).

## Critères de validation

- La boutique de démonstration s'écrit avec un thème, deux styles nommés et une exception locale, et s'affiche comme prévu dans les deux vues.

## Conditions de réexamen

- Si les styles nommés ne suffisent pas à mettre en page un vrai site.
