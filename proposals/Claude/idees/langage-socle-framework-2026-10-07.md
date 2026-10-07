# Idée gardée au chaud : le langage, le socle et le framework

- Auteur : Claude, d'après une question de Yocthan
- Date : 2026-10-07
- **Statut : idée gardée au chaud.** Ni proposition, ni essai, ni décision. Yocthan, le 2026-10-07 : « il sera juste gardé au chaud, comme pensé […] on continue d'abord sur ce qu'on était en train de travailler, et après on verra s'il sera nécessaire d'y repenser ». Rien n'est à construire, rien n'est à faire relire par Codex ou Gemini pour l'instant.

## La question

Faut-il séparer HoloCode en un langage et un framework, comme Dart et Flutter ?

## L'idée

Chez Google, Flutter est écrit en Dart : n'importe qui peut écrire un widget aussi puissant que ceux de Flutter. Dans HoloCode, ce n'est pas possible, et c'est voulu : il n'y a pas de code libre, donc les blocs de base ne peuvent pas être écrits en HoloCode ; ils vivent dans le moteur, en Rust. HoloCode aurait donc trois couches, pas deux.

| Couche | Ce que c'est | Chez Google | Rythme |
|---|---|---|---|
| **1. Le langage HoloCode** | La grammaire : blocs nommés, réglages, listes, textes, styles, règles (`On`, `When`, `Every`), valeurs (`State`), composants (`Part`), `import` | Dart | Très lent, très stable |
| **2. Le socle** | Les blocs du moteur (`P`, `Button`, `Row`, `Image`, `Input`, `Point`, `World`…) et leurs garanties : vrai HTML, accessibilité, vérifications | Le cœur de Flutter | Lent |
| **3. Le framework** | Des composants tout faits, écrits en HoloCode avec le socle (cartes, menus, pieds de page, mises en page, thèmes), qu'on importe | Material, Cupertino | Rapide ; il peut y en avoir plusieurs |

## Ce que ça apporterait

- Le langage resterait petit : un nouveau besoin deviendrait un composant du framework, pas un mot du langage.
- Plusieurs frameworks possibles, sur le même langage : pour les sites, pour les jeux, pour les mondes en 3D.
- Une communauté pourrait contribuer sans toucher au moteur : la seule voie pour faire monter un jour la note « entraide » (5 %).

## Les risques

- Deux choses à apprendre au lieu d'une : le débutant ne doit pas avoir à connaître la frontière. C'est déjà le cas : `Button(…)` et `ArticleCard(…)` s'écrivent et se restylent de la même façon.
- Découper trop tôt : aucun composant tout fait n'existe encore.

## Si on y revient un jour

1. Dans la documentation : séparer le guide en « le langage » et « les blocs du socle » ; marquer chaque mot de `NOMS.md` *langage* ou *socle*.
2. Une règle : un nouveau besoin devient d'abord un composant ; un nouveau bloc du socle, seulement si un composant ne peut pas le faire.
3. Quand trois ou quatre composants utiles existeront (la carte produit, le menu, le pied de page du site de référence) : un dossier pour le framework, et une façon de l'importer (par exemple `import "holo:cartes"` ; aujourd'hui `import` ne lit que des fichiers rangés à côté).
4. Un nom pour le framework, choisi par Yocthan, comme Flutter face à Dart.

Ce qu'on ne ferait pas : séparer le moteur en plusieurs programmes ou plusieurs dépôts.

## La question liée, déjà répondue le 2026-10-07

« Les blocs de base doivent-ils devenir des composants ? » Non : ils sont les briques avec lesquelles on construit les composants, et ils portent les garanties du moteur ; pour l'auteur, ils s'écrivent déjà comme des composants.
