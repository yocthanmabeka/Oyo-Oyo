# ADR-033 — Le site léger : le moteur n'arrive qu'au premier geste qui en a besoin

- Statut : EXPÉRIMENTATION
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 (« sur 100 %, le projet te convainc à combien ? » ; « Oui, vas-y »)
- Validation : Yocthan, le 2026-10-04, pour un essai. L'écriture est une proposition de Claude.
- Projets affectés : HoloEngine (page d'entrée)

## Contexte

Une page `.holo` arrivait déjà toute faite par le serveur : elle s'affichait aussi vite qu'une page HTML. Mais la page d'entrée téléchargeait aussitôt tout le moteur, environ 560 Ko compressés. La boutique d'exemple pesait donc 579 Ko, contre 6 Ko pour sa jumelle en HTML. Pour un site qu'on ne fait que lire, c'était cent fois trop. La promesse « ultra léger » n'était pas tenue.

## Décision (à l'essai)

1. La page d'entrée est coupée en deux : `page.html`, la page légère, et `page-moteur.js`, qui charge le moteur en Rust et prend la page en main.
2. La page légère ne télécharge rien d'autre. On lit, on défile, on suit un lien vers une autre page `.holo` (le serveur la fabrique aussi).
3. Le moteur est demandé au premier geste qui en a besoin : toucher un bloc nommé (un bouton), un lien vers un point (`#…`), entrer dans un champ, toucher le menu, zoomer (Ctrl + molette, pincer). Les touchers faits en l'attendant sont rejoués, et le menu touché s'ouvre.
4. Une page **vivante** demande le moteur tout de suite. C'est le moteur qui la marque (`data-vivant`), quand elle a une horloge (`Every`), le clavier (`Key`), des données (`Data`) ou un bloc à faire glisser (`drag`). Une page aux valeurs gardées (`keep`) le demande aussi tout de suite, mais seulement s'il y a vraiment quelque chose de gardé d'une visite précédente.
5. Le moteur arrive aussi tout de suite quand le serveur n'a pas fabriqué la page, et quand l'adresse porte `?` ou `#` (mondes, captures, mesures).

## Comparaison faite avant de choisir

| Option | Pour | Contre |
|---|---|---|
| Tout charger au départ (avant) | simple ; le premier toucher répond aussitôt | 560 Ko pour lire une page |
| Charger après l'affichage, dans les temps morts | le premier toucher répond vite | les 560 Ko partent quand même, pour rien si on ne fait que lire |
| **Charger au premier geste qui en a besoin** | une page lue pèse le poids d'une page HTML | le premier toucher attend le moteur (court en wifi, quelques secondes sur un réseau lent) |
| Couper le moteur en morceaux (la vue à plat, puis les points) | encore plus fin | gros chantier dans le moteur ; à reconsidérer plus tard |

Défaut du web évité : les sites qui envoient tout leur JavaScript avant qu'on ait rien demandé.

## Conséquences

### Positives

- Une page qu'on lit ne télécharge que son HTML.
- Rien ne change pour l'auteur : il n'écrit rien de plus.

### Négatives et risques

- Le premier toucher attend le moteur. Sur un réseau lent, l'attente se voit : le toucher n'est pas perdu, mais la réponse tarde.
- Une saisie commencée avant l'arrivée du moteur peut être effacée quand il redessine la page.
- Pas encore mesuré sur un téléphone bon marché ni sur un réseau lent.

## Critères de validation

- La boutique : au départ, seul le HTML est téléchargé ; au premier « Add », le moteur arrive et le panier passe à 1.
- Les jeux : le moteur arrive tout de suite, et ils se jouent comme avant.
- Test du moteur : `plat.rs` (`une_page_qui_bouge_seule_demande_le_moteur_tout_de_suite`).

## Conditions de réexamen

- Si l'attente du premier toucher gêne sur un téléphone ou un réseau lent.
- Quand on voudra couper le moteur en morceaux.
