# ADR-061 — Lot 9, la fin du web : toutes les touches, apparaître en descendant, le son réglé, des tailles qui suivent le visiteur, la vue points au lecteur d'écran

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : le grand tableau (`docs/01-holocode/TABLEAU-WEB.md`) : un manque « en priorité » (« en vue points, un lecteur d'écran ne voit toujours rien ») et quatre manques « utiles » (le clavier, le défilement, le son, les unités) ; Yocthan, le 2026-10-06 : « Le seul manque en priorité […] si tu le fais, tu me dis quand est-ce que je veux le voir » ; « Plus de touches du clavier. Faire apparaître un bloc quand on descend. Régler un son. Bref, oui, oui. Ça, il faut le faire » ; « tu valides déjà le tout »
- Validation : Yocthan, le 2026-10-06, en le commandant (« il faut le faire », « tu valides déjà le tout ») ; vérifié dans Chrome avant la fusion
- Projets affectés : HoloCode, HoloEngine

## Contexte

Le web de HoloCode avait cinq manques connus. Un était urgent : en vue points, la page était cachée (`visibility: hidden`), donc absente pour un lecteur d'écran. Les quatre autres étaient utiles : seulement les flèches et l'espace au clavier ; rien pour faire apparaître un bloc en descendant ; un son sans volume, sans boucle, sans arrêt ; des tailles en pixels qui ne suivaient pas le texte choisi par le visiteur.

## Décision

1. **Le clavier entier, ou presque** : `Key.enter`, `Key.escape`, `Key.a` à `Key.z`, `Key.digit0` à `Key.digit9`, en plus des flèches et de l'espace. Une lettre est celle écrite sur la touche ; un chiffre se lit par sa place (rangée du haut ou pavé numérique), donc un clavier français marche sans Maj. Jamais Tab. Une page qui écoute des lettres ou des chiffres ajoute au menu ☰ « Touches à une lettre », que le visiteur peut couper ; son choix est gardé dans son navigateur. `Key.A` est refusé avec « écris « a » » ; `Key.tab`, avec la raison.
2. **Apparaître en descendant** : `Enter(…, inView: true)`. L'entrée attend que le bloc arrive à l'écran. La page légère le guette (un `IntersectionObserver`, pas d'écoute du défilement) ; sans JavaScript, ou pour le visiteur qui demande moins de mouvement, tout se voit d'emblée.
3. **Le son réglé** : `Sound(volume: 0.4, loop: true)` ; la capacité `stop` l'arrête et le remet au début, aussi dans une règle de temps ou qui guette. Jamais de lecture à l'ouverture.
4. **Des tailles qui suivent le visiteur** : l'auteur écrit des pixels ; le moteur écrit des `rem` pour `padding`, `margin`, `width`, `max-width`, `height`, `border-radius` et l'écart des blocs `Row`, `Column`, `Grid`. Les traits, les ombres et l'écart entre les lettres restent en pixels. `height: screen` : tout l'écran, au moins (`min-height: 100dvh`, avec `100vh` pour les vieux navigateurs).
5. **La vue points au lecteur d'écran** : la page reste sous les points, invisible (`opacity: 0`) mais dans l'arbre d'accessibilité ; le dessin des points est caché au lecteur (`aria-hidden`) ; « Vue points » et « Vue web » sont annoncés (`role="status"`) ; quand le clavier arrive sur la page (Tab), la vue web revient.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi | Défaut du web évité |
|---|---|---|---|
| Lire une touche | `event.key` ; `event.code` ; les deux | **les lettres par `key`, les chiffres par `code`** : la lettre qu'on voit sur la touche ; le chiffre à sa place | sur un clavier français, `key` donne « & » pour la touche du 1 ; `keyCode`, l'ancienne façon, est abandonné |
| Les touches à une lettre | toujours actives ; seulement sur un bloc qui a le focus ; **actives, et qu'on peut couper** | un jeu marche tout de suite ; la dictée vocale reste possible | WCAG 2.1.4 : une page qui prend les lettres gêne les logiciels de dictée |
| Tab | une touche comme les autres ; **jamais** | on ne doit jamais rester coincé dans une page | le « piège au clavier » (WCAG 2.1.2) |
| Apparaître en descendant | un signal `On(Bloc.seen, …)` ; `animation-timeline: view()` en CSS ; **`Enter(…, inView: true)`** | le besoin est visuel : un mot dans l'entrée qui existe déjà ; le CSS seul ne marche pas encore partout | les bibliothèques qui écoutent le défilement (saccades, batterie) ; un bloc caché pour toujours quand le script ne vient pas |
| Le volume | **de 0 à 1**, comme l'opacité ; en pourcentage | une seule façon de dire « un peu » dans le langage | en HTML, le volume ne se règle qu'en JavaScript |
| Arrêter un son | `pause` ; **`stop`** (pause, et retour au début) | un son qu'on relance repart du début | le web n'a pas de `stop()` : il faut `pause()` puis `currentTime = 0` |
| Les unités | ajouter `rem`, `em`, `vw`, `vh`… ; **garder `px` et `%`, et écrire des `rem` à la place de l'auteur** | rien de nouveau à apprendre, et la page suit le visiteur par défaut | douze unités en CSS ; `px` qui ignore le réglage du visiteur |
| Toute la hauteur de l'écran | `100vh` ; **`height: screen`** → `min-height: 100dvh` | un mot ; le bloc grandit si son contenu est plus long | `100vh` déborde sous la barre d'adresse d'un téléphone ; une hauteur fixe coupe le contenu |
| La page en vue points | cachée ; **dessous, invisible, lisible** | le lecteur d'écran lit la même page, sans copie à tenir à jour | un `canvas` muet pour les personnes aveugles |

## Conséquences

- Une page écrite avant ce lot paraît la même avec le texte réglé par défaut (16px = 1rem) ; elle grandit seulement chez le visiteur qui a grossi son texte.
- `Enter(inView:)` ne vaut que pour une entrée ; une boucle (`Loop`) tourne toujours.
- Restent hors de ce lot : la position du défilement comme valeur, les effets de son (mélange, espace), la 3D (les listes à champs et les listes venues du serveur sont faites : ADR-051).

## Critères de validation

- Tests du moteur : les touches, `Key.A` et `Key.tab` refusés avec la raison ; le volume et la boucle écrits dans la page ; `stop` permis ; les `rem` et `height: screen` ; `inView` (et refusé dans une boucle).
- Dans Chrome (leçons 77 à 81) : P, la touche du 5 sans Maj, Entrée, Échap ; les lettres coupées depuis le menu, Entrée qui marche encore ; les cartes qui attendent loin de l'écran et entrent en arrivant, tout visible quand on demande moins de mouvement ; le son en boucle au volume écrit, puis arrêté et remis au début ; les marges qui suivent la taille du texte ; `height: screen` qui remplit l'écran ; en vue points, le titre toujours dans l'arbre d'accessibilité de Chrome, « Vue points » annoncé, Tab qui ramène la vue web.
