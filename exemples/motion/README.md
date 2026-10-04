# Le duel du motion design : HoloCode contre HTML, CSS et JavaScript

Le même film, en sept scènes : le Big Bang, le titre, trois mots, le morphing, le laboratoire des courbes, la profondeur, la signature. Écrit deux fois par Claude, chaque fois au mieux de ce que le langage permet.

- [`holocode/showreel.holo`](holocode/showreel.holo) — HoloCode, avec le mouvement ajouté le 2026-10-04 (`ADR-034`). Aucune ligne de JavaScript.
- [`web/showreel.html`](web/showreel.html) — HTML, CSS et JavaScript, sans bibliothèque : particules physiques sur un canevas, morphing SVG, cube en 3D, Web Animations, parallaxe et bouton aimanté à la souris.

Pour les voir : le serveur local (`node moteur/outils/serveur.mjs`), puis `http://localhost:8080/exemples/motion/holocode/showreel.holo` et `http://localhost:8080/exemples/motion/web/showreel.html`.

## Mesuré

| | HoloCode | HTML, CSS, JavaScript |
|---|---|---|
| Lignes écrites (sans commentaires ni lignes vides) | 206, dont 108 pour les 36 éclats écrits un par un ; **environ 100 sans eux** | 311 |
| Octets du fichier | 14 122 | 21 247 |
| Téléchargé à l'ouverture (Chrome, mesuré) | **10 Ko** : la page toute faite, le moteur n'est pas demandé | 7 Ko |
| JavaScript écrit | 0 ligne | environ 200 lignes |
| Particules | 36 formes | 520 particules physiques, traînées, onde de choc |
| Morphing | un carré qui s'arrondit | carré → rond → goutte → cœur (chemins SVG) |
| 3D | cartes qui se retournent, losange qui tourne | un cube à six faces, et toute la scène penche avec la souris |
| Réagit à la souris | non | oui |
| Moins de mouvement demandé par le visiteur | la dernière scène, arrêtée, sans rien écrire | prévu à la main dans le CSS et le JavaScript |
| Fluidité sur téléphone | pas mesurée | pas mesurée |

## Le verdict de Claude

**En puissance pure, HTML, CSS et JavaScript gagnent nettement.** Le canevas permet des centaines de particules qui obéissent à une physique, les chemins SVG se transforment en n'importe quelle forme, et tout peut répondre à la souris. HoloCode ne sait rien de cela.

**En écriture, HoloCode gagne nettement.** Pour ce que les deux savent faire (entrées, boucles, lettre à lettre, scènes enchaînées), HoloCode s'écrit en une ligne là où le web demande une image clé nommée, un délai calculé, ou du JavaScript. Un film entier tient en environ 100 lignes lisibles par quelqu'un qui ne programme pas.

| Critère | Gagnant |
|---|---|
| Ce qu'on peut montrer | HTML, CSS, JavaScript |
| Facilité, longueur | HoloCode |
| Poids | égalité (quelques Ko chacun) |
| Respect de qui demande moins de mouvement | HoloCode (automatique) |
| Réagir au visiteur | HTML, CSS, JavaScript |

Pour que HoloCode rattrape la puissance : des listes répétées (les 36 éclats en une ligne), des particules dessinées par le moteur (il sait déjà dessiner des points par milliers), des chemins à suivre, et des mouvements qui répondent au doigt.

![Le Big Bang en HoloCode](../../docs/06-journal/images/2026-10-04-motion-holocode-big-bang.png)
![Le Big Bang en HTML, CSS, JavaScript](../../docs/06-journal/images/2026-10-04-motion-web-big-bang.png)
