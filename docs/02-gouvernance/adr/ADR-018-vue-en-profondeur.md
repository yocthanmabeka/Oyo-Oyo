# ADR-018 — En profondeur, seuls les points ont de la profondeur ; le reste se lit sur un panneau

- Statut : ACCEPTÉ
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : HC-013, revue Codex du 2026-10-03 (`proposals/GPT5.6/revue-2026-10-03/`), journal du 2026-10-04
- Validation : décidé par Yocthan le 2026-10-04, en discussion avec Claude.
- Projets affectés : HoloEngine, HoloCompiler

## Contexte

`ADR-007` donne deux vues à un même fichier. Pour un `Point`, la vue en profondeur est claire : une boule dans laquelle on entre. La question restée ouverte dans `HC-013` : que deviennent un titre, un paragraphe, un bouton, une image ?

## Décision

**Option A.** Seuls les `Point` ont de la profondeur. Le texte, les boutons et les images restent lisibles, affichés sur un panneau plat posé dans le lieu. On lit comme d'habitude, et l'on plonge quand on veut.

**L'option B** (chaque bloc devient une boule) n'est pas écartée : Yocthan veut voir les deux de ses yeux. La démonstration de la boutique offrira un bouton pour passer de l'une à l'autre.

## Alternatives étudiées

- Option B d'emblée : spectaculaire, mais lire un paragraphe dans une sphère est pénible. Codex fait la même objection : un texte long, un formulaire ou une liste ont besoin d'un comportement propre.

## Conséquences

### Positives

- Fidèle à la phrase de Yocthan : « quelqu'un verra un web normal, mais pourtant c'est le métavers ».
- Un achat ou une saisie donne le même état dans les deux vues.

### Négatives et risques

- Il faut dessiner du texte lisible dans la vue en profondeur, ce que le moteur ne sait pas encore faire.

## Critères de validation

- La même boutique, dans les deux vues, avec le même état après bascule ; Yocthan compare A et B sur son téléphone.

## Conditions de réexamen

- Après la comparaison de A et B par Yocthan.
