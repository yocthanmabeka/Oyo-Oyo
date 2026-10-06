# ADR-055 — L'accessibilité vérifiée : le contraste refusé, un audit, un essai humain

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md` (point 6 : « un essai avec un lecteur d'écran, et la correction de ce qu'il trouve »)
- Validation : Yocthan, le 2026-10-06 : « je suis tes propositions, je les valide, tu as mon feu vert ».
- Projets affectés : HoloEngine, outils, exemples

## Décision

1. **Le moteur refuse un texte trop peu contrasté** : quand un même style donne la couleur du texte et celle du fond (couleurs pleines, variables comprises, dans chaque état : survol, focus, appui, sombre, téléphone), le contraste doit être d'au moins 4,5 pour 1, ou 3 pour 1 pour un grand texte (24px, ou 19px en gras), comme le demande le WCAG. Le message dit le contraste mesuré et le seuil. Un dégradé, une image ou une couleur à demi transparente ne sont pas mesurés.
2. **Un audit automatique** : `moteur/outils/accessibilite.mjs` passe axe-core sur toutes les pages d'un dossier, dans Chrome, en thème clair ou sombre, à la largeur voulue.
3. **La page d'un monde en 3D** laisse le zoom du navigateur permis, a un repère principal, un titre pour les lecteurs d'écran, et un nom pour le monde dessiné.
4. **Un essai humain** au lecteur d'écran (TalkBack, NVDA) suit `docs/01-holocode/ESSAI-LECTEUR-D-ECRAN.md` ; chaque problème trouvé devient une correction du moteur.

## Résultats (2026-10-06)

- Avant : 69 leçons sur 70 sans défaut pour axe-core ; trois défauts de contraste (le badge de la boutique, la pastille de la leçon 38, les liens de la leçon 52 en thème clair) et la page des mondes (zoom interdit, ni repère ni titre).
- Après : 0 défaut sur les 72 pages des leçons et sur tous les sites d'exemple, en thème clair et sombre, à 1000 et 390 pixels.
- Le contrôle du moteur trouve les deux badges ; il ne voit pas les liens de la leçon 52, dont la couleur et le fond sont dans deux styles différents (`A { }` et `Page { }`) : l'audit les a trouvés.

## Critères de validation

- Tests : contraste refusé, accepté pour un grand texte ou un fond plus sombre, avec une variable, dans un état, dégradé ignoré.
- L'essai humain : à faire par Yocthan, sur le Flip.
