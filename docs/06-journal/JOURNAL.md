# Journal d'évolution du projet

Ce journal raconte ce qui a été fait, ce qui a raté et ce qui a été décidé, étape par étape, avec les captures. Il est tenu par Claude après chaque étape, à la demande de Yocthan, sans le lui annoncer. La dernière entrée est en haut. Les captures sont dans [`images/`](images/).

Pour l'état courant en un coup d'œil, voir [`AGENTS.md`](../../AGENTS.md) à la racine.

---

## 2026-10-03 — Les deux limites revues ; HoloCode comparé au web, balise par balise

**Ce que Yocthan a demandé**

- Revoir les deux limites annoncées : si elles sont bonnes, les laisser ; si elles sont mauvaises, les corriger. Il en donne l'autorisation.
- Que Claude fasse à fond sa part de la mise à l'épreuve face aux balises du web classique, puis lui donne la liste de ce qu'il doit faire de son côté.

**Les deux limites**

- « Seuls les fichiers rangés à côté sont acceptés » : mauvaise limite, corrigée. `Point(inside: "https://…/jardin.holo")` mène au fichier d'un autre serveur. Le portail affiche le nom du serveur ; un tel fichier n'est lu qu'à l'ouverture du carrefour, jamais d'avance ; il passe par le vérificateur. Le serveur local envoie l'en-tête qui autorise cette lecture.
- « Rien ne borne le nombre de passages » : bonne limite pour les passages (on tourne dans une maison autant qu'on veut), mauvaise pour la mémoire. Le moteur ne garde plus que les 32 derniers fichiers lus.
- Vérifié avec deux serveurs locaux sur deux ports, donc deux sites différents pour Chrome : bouton, carrefour (le portail affiche « Ailleurs · 127.0.0.1:8081 »), franchissement, arrivée dans le jardin de l'autre serveur sans rechargement ; un cran de dézoom ramène au départ ; l'adresse en `#@…` ouvre directement le fichier distant.
- Une contrainte des navigateurs, écrite dans `ADR-022` : la barre d'adresse ne peut pas montrer l'adresse d'un autre serveur ; elle affiche le fichier de départ suivi de `#@` et de l'adresse réelle.

**La comparaison**

- [`docs/01-holocode/COMPARAISON-WEB.md`](../01-holocode/COMPARAISON-WEB.md) : HTML balise par balise, puis CSS et JavaScript, avec pour chaque ligne « fait », « exprès » ou « manque ». Verdict écrit sans détour : HoloCode sait « lire et se promener », pas encore « agir » ; le manque le plus gênant est la disposition. Suit un ordre proposé en dix rangs pour combler les manques.

**Ce qui revient à Yocthan** (liste donnée dans la conversation)

- Mesurer sur le téléphone ; écrire un vrai petit site pour sentir ce qui gêne ; relire les noms proposés ; choisir l'ordre des manques ; faire relire par Codex et Gemini.

---

## 2026-10-03 — Aller ailleurs : le lien `A`, et le point qu'on traverse d'un fichier à l'autre

**Ce que Yocthan a posé**

- Entre les mondes, pas de rechargement ni de redirection : « on traverse la porte du salon et on va au jardin, avec les mêmes jambes ». Seuls l'animation et le zoom font la transition. Le lien normal reste pour un vrai changement de site, à l'ancienne. « Porte » était une façon de parler.
- Pourquoi le lien ne s'écrit-il pas `A`, comme en HTML ? Et les listes, les `UL`, `LI` ?
- Dézoomer doit faire ressortir du monde où l'on est.
- À noter en mémoire : tout ce qui se fait doit être documenté, et avoir son élément dans le langage, pour que lui ou quelqu'un d'autre puisse reprendre le travail.

**Fait** (`ADR-022`)

- `A("texte", to: "adresse")` : le lien classique. Claude avait proposé `Link` ; Yocthan a demandé `A`, et il a raison : c'est le mot de HTML, et `link` y désigne autre chose.
- `Point(inside: "jardin.holo")` : le monde d'un point peut être un autre fichier. On y passe sans changer de page : carrefour, portail, animation ; l'adresse devient celle de l'autre fichier ; « retour » ramène. Les fichiers voisins sont lus d'avance.
- Dézoomer, quand la page est à sa taille normale, fait ressortir : au site, au fichier, ou hors du monde calculé d'où l'on venait.
- `List(ordered: true)` numérote une liste ; un élément de liste peut être un lien. Pas de `UL`, `OL`, `LI`.
- Nouvel exemple, `exemples/maison/` : un salon sombre et un jardin clair, deux fichiers.
- Vérifié avec de vrais gestes envoyés à Chrome, une marque posée dans la page prouvant qu'elle n'a pas été rechargée : bouton « Aller au jardin », le carrefour s'ouvre sur le jardin ; clic sur le portail, l'adresse devient `jardin.holo`, le fond devient clair, la page n'a pas été rechargée ; un cran de dézoom, retour à `salon.holo`.
- Guide mis à jour (liens, listes, passage entre fichiers, inventaire). Tests du cœur : 64 sur 64.

![Le carrefour du salon : au milieu, le jardin, qui est un autre fichier](images/2026-10-03-maison-2-carrefour.png)

**Erreur trouvée en vérifiant**

- Les aperçus des portails laissaient déborder leurs styles : l'aperçu du jardin (clair) repeignait celui du salon. Chaque aperçu est maintenant enfermé.

**Limites**

- Seuls les fichiers rangés à côté sont acceptés, pas le site d'un autre auteur sur un autre serveur.
- Rien ne borne le nombre de passages entre fichiers.

---

## 2026-10-03 — Chaque notion a son mot ; un carrefour rempli de mondes

**Ce que Yocthan a demandé**

- Vérifier que chaque notion apportée au site a son équivalent, son mot-clé, dans le code source : l'image, le pixel, la fragmentation, les zooms ; et chaque action : activer ou désactiver le zoom, passer d'un site à l'autre. Le son n'a pas encore été travaillé.
- Le carrefour lui plaît, mais : il n'y a que deux ou trois mondes ; il veut que des mondes remplissent toute la page. Là où le curseur se pose, le monde doit apparaître comme avant (la feuille). Une petite lumière au fond, pour y voir s'il fait noir. Et un sens de défilement à choisir : à l'horizontale, à la verticale, en diagonale, en liste ou en grille.

**Fait**

- L'inventaire est dans le guide, partie « Chaque notion et son mot » : dix-sept notions ont leur mot, huit n'en ont pas encore (le son, la vidéo, le survol, réagir au zoom par une règle, la disposition, les liens entre fichiers, les formulaires, le personnage).
- Deux mots manquaient pour ce qui existait déjà, ils sont ajoutés : `Zoom(active:)` pour permettre ou interdire le zoom, et le bloc `Portals(layout:, count:, size:, brightness:)` pour le carrefour. La page gagne la capacité `portals` : `On(Map.tap, effect: Shop.portals)` ouvre le carrefour par une règle.
- Le carrefour remplit la fenêtre : les sites écrits dans le fichier d'abord, avec leur contenu, puis des mondes calculés à partir d'une graine, jusqu'à `count`. Chaque monde calculé est une boule de lumière de sa couleur ; un clic l'ouvre en profondeur (le Big Bang), à l'adresse `fichier.holo#~graine`, et « Retour » ramène au site.
- Là où le curseur se pose, le monde grandit et devient une feuille lisible.
- Le fond est éclairci par une lueur (`brightness`).
- Quatre dispositions : `grid`, `row` (on défile de gauche à droite), `column` (de haut en bas), `diagonal`.
- Vérifié avec de vrais gestes envoyés à Chrome : douze portails ; le survol agrandit ; un clic sur un monde calculé donne l'adresse `#~13044733080473193766` et le moteur dessine ; « Retour » ramène au site.
- Tests du cœur : 62 sur 62.

![Le carrefour rempli de mondes ; le curseur est sur le monde jaune](images/2026-10-03-carrefour-grille.png)

**Limites**

- Les mondes calculés ne contiennent pas de site : ce sont des univers de points. Y poser des sites écrits par d'autres personnes demande les liens entre fichiers.
- Les dispositions `row`, `column` et `diagonal` sont lues et appliquées, mais Claude n'a capturé que la grille.

---

## 2026-10-03 — Glisser déplace la page pendant tout le zoom

**Ce que Yocthan a relevé**

- Depuis le zoom ordinaire, on ne pouvait plus déplacer le site en glissant, à gauche ou à droite, comme on le faisait en vue points. Il ne savait plus comment se rapprocher d'un point : « ça devient n'importe quoi ».

**La cause**

- Claude avait fait deux zooms qui se suivent (la page vivante jusqu'à × 4, les points ensuite) avec deux façons de se déplacer : la barre de défilement d'abord, le glissement ensuite. Et en revenant des points, la page reprenait à l'endroit d'où l'on était parti, pas à celui où l'on était arrivé.

**Fait**

- Dès que la page est grossie, glisser la déplace, dans tous les sens : le geste est le même du début à la fin du zoom. Une main l'indique. Un double clic sélectionne toujours un mot.
- En revenant de la vue points, la page vivante reprend là où l'on se trouvait.
- Vérifié avec de vrais gestes envoyés à Chrome (voir la pull request).

---

## 2026-10-03 — Le carrefour à portails, le zoom ordinaire avant les points, la limite des niveaux

**Retour de Yocthan sur la boucle**

- Le bref noir à l'ouverture d'un site dérange. Son idée : à la place, des points gros et proches, prêts à être cliqués, « comme les portails de Strange dans Avengers Endgame » : des mondes déjà là, sur toute la page, qui incitent à changer de monde. C'est ce qu'il appelait la « roadmap métaverse ».
- Pas de fond noir quand le site est clair : il faut suivre le style du site.
- Les points autour de la feuille ne doivent pas être seulement calculés par la graine.
- Mettre une limite au nombre de niveaux.
- Un visiteur est habitué à zoomer pour lire, à sélectionner, à copier. Le zoom doit d'abord rester normal ; la « métaversification » ne commence qu'à partir d'une certaine profondeur.
- Question : une photo, une vidéo se décomposent-elles de la même façon ?

**Fait**

- **Le carrefour.** Entrer dans un point qui contient un site n'ouvre plus une feuille dans un monde noir, mais des portails ronds posés sur le fond du site où l'on est. Chaque portail montre le site où il mène, en petit. Autour du portail visé : les autres sites contenus dans la page, le site où l'on est, celui d'où l'on vient. Ce sont de vrais sites, écrits dans le fichier, pas des points tirés d'une graine. Un bouton « Carrefour » l'ouvre à tout moment.
- **Plus de noir.** Un clic sur un portail le fait grandir jusqu'à remplir la fenêtre, et il devient le site, sans recharger la page. Le fond de la fenêtre prend la couleur du site.
- **Le zoom ordinaire d'abord.** Ctrl + molette grossit d'abord la page vivante, jusqu'à `Points(after: 4)` : le texte reste du texte. Au-delà, ses pixels deviennent des points. En revenant, on retrouve la page grossie, puis sa taille normale. Pendant le zoom ordinaire, le moteur ne dessine rien.
- **La limite.** `Zoom(levels: 8)` : un fichier qui emboîte plus de sites est refusé, avec la ligne du point de trop.
- Vérifié avec de vrais gestes envoyés à Chrome : un cran de zoom, page vivante grossie 1,9 fois, zéro image dessinée ; trois crans, passage aux points ; retour à la taille normale en dézoomant. Zoom sur le pixel rose, clic : le carrefour s'ouvre sur « Secret » ; clic sur le portail : adresse `#Secret`, sans rechargement.
- Guide et `ADR-021` mis à jour. Tests du cœur : 60 sur 60.

![Le carrefour : trois portails vers trois sites](images/2026-10-03-pixel-2-dedans.png)

**Réponse donnée sur la photo et la vidéo**

- Une photo dans la page se décompose déjà comme le reste : ses pixels font partie de l'image de la page. Une vidéo, pas encore : il faudrait redécomposer chaque image, vingt-cinq fois par seconde. C'est faisable pour la partie visible à l'écran, mais c'est un chantier à mesurer.

**Retiré**

- La « vue personnage » (la page en feuille dans un monde noir) et la feuille d'un site dans le monde de son point : le carrefour les remplace. Yocthan avait précisé que la vue personnage vaut pour un jeu, où le monde est l'environnement du joueur.

**Erreurs en route**

- Le zoom ordinaire, fait d'abord avec la propriété `zoom` du CSS, déplaçait la mise en page : ce qui était sous la souris n'y restait pas. Refait avec un vrai agrandissement.
- À la fusion de la pull request n° 33, une coupure de réseau a masqué le résultat des vérifications ; la fusion est partie quand même. Contrôlé juste après : tout était vert. À l'avenir, vérifier avant, dans une commande séparée.

**Limites**

- En vue points sur un site clair, les points sont lumineux sur fond noir : le blanc de la page devient des points blancs.
- Les portails ne montrent que les sites à un pas de distance ; il n'y a pas encore de carte d'ensemble.
- La première entrée en vue points prend un instant (la page est redessinée dans une image).

---

## 2026-10-03 — La boucle : un site dans un pixel, dans un site, dans un pixel…

**Retour de Yocthan sur le site planté dans un pixel**

- Le site caché apparaît bien, mais il s'arrête à la vue personnage. Il veut qu'un deuxième clic, ou un zoom, l'agrandisse jusqu'à sa taille normale, et qu'on retrouve alors le même mécanisme que pour le site d'origine : zoomer, voir les pixels, cliquer, entrer. « La boucle va continuer jusqu'à l'infini. »
- Il a aimé les points derrière la feuille. Il les veut répartis sur toute la fenêtre plutôt qu'un fond noir : l'utilisateur cliquera sur d'autres sites, d'autres mondes. « Ce sera vraiment le multivers. »

**Fait**

- Le monde d'un point s'ouvre en grand comme une page. Un deuxième clic sur la feuille, ou un zoom qui la grossit assez, et le site caché prend toute la fenêtre : c'est alors un site comme un autre, avec ses boutons, sa vue points, sa vue personnage.
- Un `World` accepte `pixels:` : un site peut en contenir un autre, qui en contient un autre. Le site de Yocthan a maintenant trois niveaux (son site, « Le site caché » dans un pixel rose, « Le trésor » dans un pixel doré du site caché).
- Chaque site a son adresse : le fichier, puis `#` et le chemin des points, comme `mon-site.holo#Secret/Tresor`. Le bouton « retour » du navigateur remonte d'un niveau, et une règle `leave` aussi.
- Les points du monde entourent la feuille sur toute la fenêtre : la feuille est plus petite, et l'on part d'un peu plus près.
- Vérifié avec de vrais gestes envoyés à Chrome : zoom sur le pixel rose, clic, feuille ; clic sur la feuille, adresse `#Secret` et site en grand ; zoom sur le pixel doré, clic, feuille du trésor ; clic, adresse `#Secret/Tresor` ; bouton « Remonter », adresse `#Secret`.
- Guide mis à jour. Tests du cœur : 59 sur 59.

![Le site caché ouvert en grand, avec son propre pixel doré à droite](images/2026-10-03-pixel-3-en-grand.png)

![Le troisième site, dans un pixel du site caché](images/2026-10-03-pixel-4-tresor.png)

**Limites, dites à Yocthan**

- Ouvrir un site en grand recharge la page : il y a un bref noir entre la feuille qui grossit et le site en grand.
- Les points autour de la feuille sont ceux que la graine calcule : on peut y entrer (c'est le Big Bang), mais ils ne contiennent pas de site écrit. Y poser de vrais sites est le chantier A.
- Le nombre de niveaux n'est pas limité : le garde-fou que Yocthan a évoqué reste à décider.

---

## 2026-10-03 — Un site planté dans un pixel de la page

**Ce que Yocthan a demandé**

- Le chantier A (voir la source d'un point, écrire dans un point) est confirmé en premier. Pour la police du rendu natif : « fais ce qui est le mieux », donc Noto Serif.
- Il a d'abord voulu apprendre à écrire : Claude lui a fait un site de départ à lui, `exemples/mon-site/mon-site.holo` (dans son dossier, hors GitHub), et une leçon en cinq règles.
- Sur la vue personnage, il a précisé : elle vaut pour un jeu, où le monde est l'environnement du joueur (Trevor, son téléphone et sa ville sont dans un point dédié à GTA). Pour un site, il veut voir ceci : choisir un pixel de la page, y planter un site d'une autre couleur, zoomer, cliquer dessus, s'en approcher, et y entrer ; et ce site du dedans doit se voir en vue personnage, puisqu'il est à l'intérieur d'un point. « Tu le fais, et après on en parle. »

**Fait**

- `Page(pixels: [ Point(name:, above:, color:, inside: World(...)) ])` : un point planté dans un pixel de la page, juste au-dessus du bloc nommé par `above`, à l'extrémité droite. Au repos il fait un pixel. En vue points il grossit avec les autres ; un clic dessus fait s'approcher, puis entrer.
- Le site contenu dans un point (celui-ci, mais aussi l'atelier de la boutique) ne s'affiche plus sur un panneau en bas de l'écran : c'est une feuille posée dans le monde du point, qui tourne et grandit avec lui, comme la page en vue personnage.
- Vérifié avec de vrais gestes envoyés à Chrome sur le site de Yocthan : trois crans de zoom sur le pixel, un clic, et l'on se retrouve devant « Le site caché », rose sombre, dans son monde.
- Le guide a une nouvelle partie, avec un exemple relu par le test. Tests du cœur : 58 sur 58.

![Le pixel rose, après trois crans de zoom](images/2026-10-03-pixel-1-zoom.png)

![Après le clic : le site caché, posé dans le monde du point](images/2026-10-03-pixel-2-dedans.png)

**Provisoire, dit à Yocthan**

- `pixels:` et `above:` sont une écriture d'essai, faite pour voir l'effet. On ne sait placer un point que « au-dessus d'un bloc, à droite ».
- Les millions d'autres points de la page restent calculés : on ne peut rien écrire dedans.
- L'approche est un zoom rapide, pas encore une vraie traversée ; en sortant du site caché on revient à la page entière, pas à l'endroit où l'on avait zoomé.

---

## 2026-10-03 — Le guide de l'auteur : comment écrire en `.holo`

**Ce que Yocthan a relevé**

- « Supposons que je veux écrire en `.holo`. Qu'est-ce que je fais ? Je n'ai pas encore vu la documentation. » Il avait raison : le langage avançait, mais rien n'expliquait à un auteur comment s'en servir.
- Le fichier du Big Bang n'avait pas bougé depuis le premier jour.
- Il ne voyait pas à l'écran les fichiers sur lesquels on travaille.

**Fait**

- [`docs/01-holocode/GUIDE.md`](../01-holocode/GUIDE.md) : le guide de l'auteur. Onze parties, de la première page aux réglages de vue, avec un aide-mémoire et la liste de ce qui n'existe pas encore. Ses sept exemples sont relus par un test du moteur : un exemple qui ne marcherait plus ferait échouer le test.
- `moteur/mondes/big-bang.holo` : chaque ligne est maintenant expliquée, et les deux réglages facultatifs (`color`, `palette`) sont montrés en commentaire, prêts à être essayés. Le comportement du Big Bang, validé par Yocthan, n'est pas changé.
- Le guide, la boutique et le Big Bang sont ouverts dans le VS Code de Yocthan.
- Yocthan a donné son accord pour le rendu natif des lettres (le moteur dessine lui-même les lettres, sans passer par une image). Police proposée : Noto Serif, à défaut d'autre choix de sa part. Rien n'est commencé.
- Tests du cœur : 57 sur 57.

**Erreur de méthode, à retenir**

- Claude a ajouté au langage pendant une journée entière sans écrire la notice à mesure. À l'avenir : chaque ajout au langage met à jour le guide dans la même pull request.

---

## 2026-10-03 — Sobriété : le moteur ne dessine que lorsqu'on bouge ; la question du « natif »

**Ce que Yocthan a relevé**

- Il voit beaucoup de fichiers `.png` liés aux images quand Claude travaille. Il veut un langage vraiment natif, qui ne passe pas par des images, même si tout peut devenir image.
- Vérifier la consommation : l'énergie ne doit être dépensée que lorsqu'on entre dans la « métaversification », pas quand on lit un site.
- Il ne savait plus dans quel fichier se trouve la boutique : `exemples/boutique-comparee/boutique.holo`.

**Mesuré, puis corrigé**

- Sur le site normal, le moteur ne fait rien : aucune zone de dessin n'existe, zéro image dessinée. C'était déjà le cas.
- En vue points, le moteur redessinait soixante fois par seconde même quand rien ne bougeait. Il s'arrête maintenant de lui-même après quelques images immobiles, et repart au premier geste. Le suivi côté page fait de même.
- Mesure avec de vrais gestes envoyés à Chrome, sur `boutique.holo` : site normal pendant 5 s, 0 image ; trois crans de zoom, 12 images ; puis 8 s sans toucher, toujours 12 ; un cran de plus, 16. Avant la correction, 8 s d'attente coûtaient environ 480 images.
- Un monde (le Big Bang) continue de se dessiner en permanence : ses points pulsent. La pause existante le coupe quand l'onglet est caché.

**Sur les images**

- Les `.png` de `docs/06-journal/images/` sont les captures que Claude prend pour vérifier son travail et tenir ce journal. Le langage ne s'en sert pas.
- L'essai sur image fixe (`mosaique.html` et son `boutique.png`) est supprimé : il n'avait plus de raison d'être depuis que la vue points part du fichier `.holo`.
- Il reste un vrai passage par l'image, dit à Yocthan : pour obtenir les points, la page est redessinée en mémoire dans une image dont on lit les pixels. Aucun fichier n'est écrit, mais le moteur ne sait pas quel point appartient à quelle lettre. Pour que ce soit natif, le moteur doit dessiner lui-même les lettres et les formes à partir des blocs. C'est un chantier à part, proposé à Yocthan ; rien n'est construit.

---

## 2026-10-03 — Ce que fait le moteur s'écrit maintenant dans le fichier : `Zoom`, `Points`, `Relief`

**Ce que Yocthan a relevé**

- Tout ce qu'on a ajouté n'apparaît pas dans les fichiers `.holo`. Il regardait `big-bang.holo`, six lignes : « on a programmé tout un monde, mais je ne vois même pas les traces de ces mondes-là. Comment les gens vont-ils programmer ? » Il avait raison : les seuils, la grille, le relief et les limites étaient des nombres écrits dans le moteur.
- En dézoomant trop fort, le site rétrécissait et sortait de son cadre.
- Il faut des garde-fous des deux côtés (zoomer sans fin, dézoomer sans fin), programmés par l'auteur. Réduire le site jusqu'à la taille d'un pixel doit être possible, mais seulement si l'auteur l'active.

**Fait**

- Trois blocs s'écrivent dans la page (`ADR-021`) : `Zoom(max:, shrink:)`, `Points(size:, fragment:, grid:, depth:, density:)`, `Relief(height:, tilt:)`. Le moteur les lit et s'y tient. Deux unités de plus : `px` et `deg`. Chaque réglage a des bornes que l'auteur ne peut pas dépasser ; une valeur hors bornes est refusée avec sa ligne.
- `exemples/boutique-comparee/boutique.holo` écrit ces réglages, avec un commentaire par ligne. Nouveau fichier `exemples/zoom/reduire.holo`, qui les change : la page s'y réduit jusqu'à devenir un seul point.
- Le rétrécissement du site : c'était le zoom de Chrome lui-même (Ctrl + molette vers l'arrière), que la page ne retenait que dans certains cas. Elle le retient maintenant toujours. Sans `shrink: true`, dézoomer sur la page ne fait rien.
- Vérifié avec de vrais gestes envoyés à Chrome : trois crans en arrière sur la boutique ne changent rien ; les mêmes sur `reduire.holo` réduisent la page ; douze crans en font un point.
- Tests du cœur : 56 sur 56.

**Réponse donnée à Yocthan sur les six lignes**

- Un fichier `.holo` dit ce que l'on veut, pas comment le faire : `big-bang.holo` dit « un point, graine 1, douze fragments », et le moteur (environ 3 500 lignes de Rust) fait le reste. C'est voulu, pour qu'un non-programmeur puisse écrire. Mais il avait raison sur le fond : ce qui se règle doit se voir dans le fichier.

**Limites**

- Les noms sont une proposition de Claude, à confirmer par Yocthan.
- Les seuils de zoom d'un `Point` seul (le Big Bang) sont encore dans le moteur ; la vue personnage et la vue « roadmap » ne sont pas décrites dans le langage.

---

## 2026-10-03 — La mosaïque gagne la profondeur (l'axe Z) et de belles lettres

**Retour de Yocthan sur la mosaïque**

- Il a beaucoup aimé (« je suis à l'extase »). La pull request n° 28 est fusionnée.
- Deux défauts : on ne voit qu'en 2D, il manque le Z ; et les lettres sont « quasiment horribles », presque illisibles.

**Fait**

- **Le Z.** Chaque point a maintenant trois coordonnées. De face, la page reste plate comme une feuille, et les points sont exactement à la place des pixels. Quand on la fait tourner (bouton « Tourner », bouton droit de la souris, ou Maj), on la voit de biais, en perspective, et ce qui est lumineux se soulève au-dessus du fond : les lettres deviennent des objets en relief. Les points nés d'un morcellement ne sont pas tous à la même hauteur. L'image ordinaire, tant qu'on la voit, tourne de la même façon : c'est la feuille de papier dans l'espace.
- **Les lettres.** Trois causes de laideur, trois corrections. L'image d'origine portait des franges bleues et orange (le lissage coloré des écrans plats) : elle est reprise sans. Elle était trop grossière : elle est reprise deux fois plus fine (2560 × 1440, soit 3 686 400 points au repos). Et en se morcelant, chaque pixel devenait un carré aux couleurs tirées au hasard : la couleur d'un point né d'un morcellement est maintenant celle de l'image à cet endroit précis, fondue entre les pixels voisins, et sa graine ne la déplace que peu aux premiers niveaux.
- Tests du cœur : 52 sur 52, dont : le zoom garde sous le doigt le même pixel même quand la page est de biais ; un aller et retour entre la page et l'écran retombe au même endroit.

![Les lettres, un point par pixel](images/2026-10-03-mosaique-5-lettres.png)

![La même vue de biais : les lettres se soulèvent](images/2026-10-03-mosaique-6-de-biais.png)

- **Une seule adresse, une seule page.** Yocthan a relevé deux choses : la barre d'adresse montrait `mosaique.html` au lieu du fichier `.holo`, puis, après une première correction, que `boutique.holo?` et `mosaique.html` ne se comportaient pas de la même façon, avec un `?` en trop. Il avait raison : c'étaient deux portes séparées. Il n'y en a plus qu'une. On ouvre `http://localhost:8080/exemples/boutique-comparee/boutique.holo` : c'est le site normal, vivant. On zoome dessus (Ctrl + molette, ou pincer) : la même page devient des points, sans changer d'adresse ni recharger. On dézoome jusqu'au bout : on retrouve le site normal. C'est le zoom qui déclenche la vue, comme il le demandait ; un bouton « Vue points » reste pour les écrans tactiles.
- Pour y arriver, le moteur fabrique la page, puis on la redessine dans une image (un SVG qui la contient, possible parce que c'est le moteur qui l'a fabriquée) dont chaque pixel devient un point. Ce n'est plus une image fixe prise d'avance : si l'on change le fichier, les points changent.
- Vérifié avec de vrais gestes envoyés à Chrome par l'outil de capture (variable `HOLO_GESTES`) : cinq crans de Ctrl + molette sur le titre font apparaître les points ; deux crans en avant puis quatre en arrière ramènent au site normal.

![Après cinq crans de Ctrl + molette sur le titre de boutique.holo](images/2026-10-03-points-3-ctrl-molette.png)

**Erreur en route**

- Le relief était compté en pixels de l'image : en zoomant très profond, il devenait immense et le moteur cherchait des points sur des milliards de cases. Un test est resté bloqué. Le relief est maintenant borné à 80 pixels d'écran.

**Limites**

- Le relief vient de la lumière : ce qui est clair se soulève. Sur une page à fond blanc et texte noir, ce serait l'inverse de ce qu'on veut. Il faudra partir de la page elle-même, où le moteur sait ce qui est une lettre.
- On tourne la page jusqu'à 52° environ, pas au-delà : on ne peut pas encore passer derrière ni se placer entre deux lettres.
- Dans la vue points, la page est une image d'elle-même : ses boutons ne répondent pas tant qu'on n'est pas revenu au site normal.
- Ctrl + molette est le geste de zoom de Chrome : la page l'intercepte. Sur un écran tactile, le premier pincement n'est pas encore capté ; il faut le bouton « Vue points ».
- Tourner la page à la souris et les gestes au doigt ne sont pas essayés par Claude.

---

## 2026-10-03 — VS Code reconnaît le langage : extension HoloCode

**Fait**

- Yocthan a demandé que VS Code reconnaisse le langage, et que ce soit le fichier `.holo` qu'on lance, pas un fichier `.html`.
- `outils/vscode-holocode/` : une extension VS Code. Elle colore les fichiers `.holo` (blocs, paramètres, styles, textes, unités, couleurs, commentaires) et ajoute un bouton ▶ en haut à droite de l'éditeur (ou `Ctrl+Alt+H`) qui enregistre le fichier et l'ouvre dans le navigateur à sa propre adresse.
- Le paquet se fabrique avec un script Python (`empaqueter.py`), sans rien installer d'autre. L'extension est installée dans le VS Code de Yocthan (`holoverse.holocode`).
- Le serveur local accepte une variable `HOLO_DEPOT` : il affiche alors les fichiers `.holo` d'un autre dossier que celui où le moteur a été construit. Il est relancé ainsi, pour que ce que Yocthan écrit dans son dossier principal soit ce qui s'affiche.

**Limites**

- Le bouton ▶ n'ouvre que les fichiers de `exemples/` et de `moteur/mondes/`.
- Les erreurs ne sont pas soulignées dans l'éditeur ; elles s'affichent dans le navigateur. Les souligner demandera de brancher le vérificateur du moteur sur VS Code (c'est le `holo check` de la liste des manques de Codex).
- La coloration n'a pas été contrôlée par Claude à l'écran : seules la validité des fichiers et l'installation ont été vérifiées.
- L'essai de mosaïque reste une page à part (`mosaique.html`), parce qu'il part d'une image et non d'un fichier `.holo`.

---

## 2026-10-03 — La mosaïque : la page est faite de points, un par pixel

**Ce que Yocthan a précisé après avoir vu la vue personnage**

- La vue lui plaît, mais passer de « vue web » à « vue personnage » par un bouton donne l'impression de changer de lien, pas d'être dans un métavers. À revoir.
- Il ne veut pas de points lumineux décoratifs au fond. **Tous les éléments de la page sont des points** : le titre « My shop », le fond, les couleurs. Le site est une image découpée en pixels, et chaque pixel est un point, rangé exactement à sa place. Le nombre de points est celui des pixels visibles. Chaque point contient d'autres points ; c'est le zoom qui les révèle.
- Une lettre est traversée par plusieurs pixels : elle contient donc plusieurs points, donc plusieurs mondes, et elle peut aussi être vue comme un objet (son exemple de la personne en casque placée entre le « 3 » et le « D »). Le monde d'un point doit donc se rattacher au caractère auquel il appartient.
- La vue « roadmap » : une carte qui répertorie les mondes actifs à un moment donné, sous forme de gros points, comme la carte des niveaux d'un jeu. Plus tard.
- Le zoom du Big Bang, déjà validé, reste tel quel ; il faut le « redisposer » pour que ce soit performant. Cela servira aussi pour la vidéo et pour des mondes générés par une IA.

**Fait**

- Pull request n° 27 (vue personnage) fusionnée.
- `moteur/src/mosaique.rs` : une image vue comme un ensemble de points. Au repos, un point fait exactement un pixel et l'image ordinaire suffit. En zoomant, les pixels grossissent et deviennent des points lumineux alignés ; au-delà de 40 pixels, chaque point se morcelle en une grille de 4 × 4, et ainsi de suite jusqu'à vingt niveaux. La graine d'un point se calcule à partir de sa place : rien n'est stocké.
- Les calculs donnés à Yocthan se vérifient : on ne dessine jamais plus de points que l'écran ne peut en montrer (un test le contrôle sur soixante zooms successifs). Sur l'image de 1280 × 720, soit 921 600 points au repos, il y a entre 4 000 et 12 000 points à l'écran quel que soit le niveau.
- Essai visible : `http://localhost:8080/mosaique.html`, sur une image fixe de la boutique. Molette ou pincement pour zoomer, glisser pour se déplacer.
- Les points décoratifs autour de la feuille, en vue personnage, sont éteints.
- Nouvel outil `moteur/outils/capturer.mjs` : prend une capture après une attente en temps réel. Les captures précédentes étaient prises trop tôt, avant le premier dessin.
- Correction : un titre dont le style fixait les marges n'était plus centré dans la page. Le centrage vient maintenant du conteneur.
- Tests du cœur : 49 sur 49.

![Les pixels du titre deviennent des points](images/2026-10-03-mosaique-2-points.png)

![Chaque point se morcelle en grille](images/2026-10-03-mosaique-3-morcele.png)

![Quatre niveaux plus bas](images/2026-10-03-mosaique-4-profond.png)

**Limites, dites à Yocthan**

- C'est une image fixe : Chrome ne donne pas au moteur les pixels d'une page vivante. Les boutons ne marchent donc pas dans cet essai.
- Les mondes des points ne sont pas encore rattachés aux caractères : avec une image, le moteur ne sait pas quel pixel appartient à quelle lettre. Il faudra partir de la page elle-même.
- On zoome et on se déplace à plat. Regarder la page de biais, et la vue « roadmap », ne sont pas faits.
- Le bord des lettres montre des points bleus et orange : c'est le lissage des polices de l'écran, présent dans l'image d'origine.

---

## 2026-10-03 — La vision se précise (le téléphone de Trevor) ; première vue « personnage »

**Ce que Yocthan a précisé**

- Par défaut, on est devant un site web normal. Le métavers vient en plus.
- Un site est comme une feuille de papier : plate, mais c'est un vrai objet dans un espace en 3D.
- Dans GTA, quand Trevor regarde son téléphone, le menu est en 2D à l'intérieur du monde en 3D. Pareil ici : si l'utilisateur est un **personnage**, il voit le site comme une surface posée dans le monde ; sinon il voit le site en 2D classique. Trois situations : il utilise le web (vue classique), il joue son propre personnage (par ses yeux), il joue à un jeu (personnage vu de dos).
- La peau paraît lisse ; en zoomant, on découvre les microbes. Le zoom révèle les mondes.
- Une option « rendre en 3D » donnera du volume à un élément important, comme des lettres entre lesquelles on peut se placer.
- Il a ainsi corrigé la proposition de Claude (tout piloté par le zoom, la page qui « devient » un lieu) : rien ne change de nature, c'est le même objet vu d'une autre place. Et il a relevé, avec raison, que dans la première démo la page disparaissait pour laisser place à un autre écran.

**Fait**

- Yocthan a confirmé qu'il voit bien les points en 3D dans l'atelier : la pull request n° 26 est fusionnée.
- Feu vert pour un premier pas, à corriger ensuite petit à petit. `moteur/web/page.html` a maintenant un bouton « Vue personnage » : la même page, sans être rechargée, devient une feuille posée au centre d'un monde, avec les points lumineux autour. Elle tourne avec le monde quand on le fait tourner, grandit quand on s'approche, et reste une vraie page : le texte se lit, le bouton marche. « Vue web » ramène au site classique.
- Moteur : `Navigation::surface` dit où poser une surface plate dans le monde ; `changer_de_monde` passe d'un monde à l'autre sans relancer la carte graphique. Tests du cœur : 42 sur 42.
- Un sommaire, `http://localhost:8080/accueil.html`, ouvre chaque démonstration à part.

![La boutique vue en personnage : une feuille dans le monde](images/2026-10-03-boutique-holo-personnage.png)

- Yocthan a demandé de lancer le fichier `.holo` lui-même dans Chrome, et non `page.html`. Le serveur local ouvre maintenant un `.holo` par sa propre adresse (`http://localhost:8080/exemples/boutique-comparee/boutique.holo`) : quand le navigateur demande le fichier pour l'afficher, il reçoit la porte d'entrée du moteur, qui va chercher le fichier. C'est le rôle que tiendra plus tard un navigateur qui sait lire le `.holo`.

**Erreur en route**

- Le serveur renvoyait encore le texte brut : l'outil qui a écrit la modification avait transformé deux `` en caractères invisibles dans le test. Trouvé en lançant une copie du serveur avec un affichage de contrôle.

**Provisoire, à décider**

- Le monde où la page est posée n'est écrit nulle part dans le fichier : sa graine est tirée du nom de la page. Le langage ne sait pas encore dire « un monde qui contient une page ».
- C'est une vue par les yeux (première personne). Le personnage vu de dos demande un corps à dessiner : chantier à part.
- La feuille est toujours dessinée devant les points, même ceux qui devraient passer devant elle.

**Ce qui n'est pas vérifié**

- Les captures montrent l'affichage des trois états. Les gestes (faire tourner le monde, zoomer, passer d'une vue à l'autre plusieurs fois) n'ont pas été essayés par Claude dans un vrai navigateur. La pull request reste ouverte jusqu'au retour de Yocthan.

**Note sur les captures**

- Les captures sans écran sortaient noires parce qu'elles étaient prises trop tôt : avec un délai de 15 secondes de temps simulé, la vue en profondeur apparaît.

---

## 2026-10-03 — Proposition : le même web en 3D, et la comparaison avec les concurrents

**Fait**

- Yocthan a demandé de s'inspirer des éléments de HTML, CSS et JavaScript pour créer ceux de HoloCode, de voir comment chacun se représente en 3D, d'innover, et de comparer avec les frameworks JavaScript et les autres langages. Claude a écrit une proposition : [`proposals/Claude/web-en-3d-2026-10/`](../../proposals/Claude/web-en-3d-2026-10/README.md). Rien n'y est décidé, rien n'est ajouté au moteur.
- Idée principale, « option C » : le plan de la page (`H1`, `H2`, `H3`) devient la profondeur. Chaque titre avec son contenu devient un point ; zoomer révèle les sous-titres. La règle d'`ADR-020` qui interdit de sauter un niveau y trouve son utilité.
- Deuxième idée : le lien est une porte. `Link(to: "autre.holo")` est un lien à plat et un point dans lequel on entre en profondeur.
- Comparaison écrite sans embellir : les signaux (Solid, Vue, Svelte) et l'arbitre (Elm) existent ailleurs ; A-Frame déclare déjà de la 3D en balises ; VRML a échoué dans les années 1990 ; Elm est juste sur le fond et peu adopté. Ce qui est propre à HoloCode est l'assemblage : une description pour deux vues, la profondeur tirée du sens, un monde dans une graine, tout vérifié avant d'exécuter.

- Yocthan veut observer le site et le point séparément avant de donner sa réflexion. Ajout d'un sommaire (`http://localhost:8080/accueil.html`) qui ouvre chaque démonstration à part : la boutique en HoloCode, la boutique en web, le Big Bang seul, et le point de l'atelier seul (`moteur/mondes/atelier.holo`).

**En attente**

- La pull request n° 26 (vue à plat et entrée dans un point) n'est pas fusionnée : Yocthan n'a pas encore dit s'il voit les points en 3D derrière le panneau.

---

## 2026-10-03 — La boutique en HoloCode s'affiche dans Chrome : vue à plat et entrée dans le point

**Fait**

- Yocthan a demandé à voir les deux versions tourner dans Chrome. La version web tournait déjà ; pour la version HoloCode, il a fallu construire l'affichage. C'est l'étape « vue à plat » prévue, avancée d'un cran.
- `moteur/src/plat.rs` : d'un fichier `.holo` vérifié, le moteur fabrique une page web ordinaire, HTML et CSS (`ADR-011`). L'auteur n'en écrit pas. Tout ce qu'il écrit est échappé : aucun texte ne peut devenir du code.
- `moteur/src/regles.rs` : les noms en double, les règles (`On(Open.tap, effect: Workshop.enter)`), les capacités inconnues et les budgets dépassés sont maintenant vérifiés. La page d'accueil (`moteur/web/page.html`) ne décide de rien : chaque toucher est envoyé au moteur, qui répond par les effets demandés.
- Entrer dans le point ouvre la vue en profondeur du point (le Big Bang, avec sa graine, sa couleur et sa palette) ; le contenu du monde se lit sur un panneau devant (option A d'`ADR-018`). « Back to the shop » ramène à la page et met la vue en profondeur en pause.
- `color` et `palette` d'un `Point` sont lus par le moteur (`ADR-017`) ; sans couleur imposée, la graine décide, dans les deux vues.
- Le serveur local sert aussi `exemples/`. Adresses : `http://localhost:8080/page.html` (HoloCode) et `http://localhost:8080/exemples/boutique-comparee/web/index.html` (web).
- Tests du cœur : 40 sur 40. Les seize cas refusés de la suite de conformité sont maintenant tous refusés par le moteur.

![La boutique en HoloCode, vue à plat](images/2026-10-03-boutique-holo-page.png)

![L'atelier : le panneau lisible devant la vue en profondeur](images/2026-10-03-boutique-holo-monde.png)

**Ce qui n'est pas vérifié**

- Sur les captures prises sans écran, la zone de dessin 3D reste noire (le Big Bang seul aussi, avec cette méthode de capture) : les points derrière le panneau n'ont pas pu être contrôlés par Claude. À Yocthan de dire ce qu'il voit dans son Chrome.

**Limites**

- Le point `Storeroom`, écrit dans le monde de l'atelier, apparaît sur le panneau mais on ne peut pas encore y entrer : la vue en profondeur ne connaît que les points nés de la graine.
- Le fond donné au monde par son style est caché par la zone de dessin 3D.
- Une seule vue en profondeur par page pour l'instant.
- Du Markdown, seuls le gras et l'italique sont rendus.
- L'option B d'`ADR-018` (chaque bloc devient une boule) n'est pas faite.

**Erreurs en route**

- Premier essai : zoom de départ à 3,7, qui faisait entrer dans un enfant du point au lieu du point lui-même. Corrigé à 2.
- Le thème de la page n'atteignait pas le monde intérieur (police différente) : le monde est maintenant rangé dans la page.

---

## 2026-10-03 — La même boutique écrite deux fois : HoloCode, et HTML, CSS, JavaScript

**Fait**

- Avant d'afficher la boutique, Yocthan a demandé un même exemple dans les deux écritures, avec tout le vocabulaire du langage, tel qu'on s'en servirait pour un métavers. Il est dans [`exemples/boutique-comparee/`](../../exemples/boutique-comparee/README.md).
- `boutique.holo` : 62 lignes utiles. La version web : 178 (36 de HTML, 61 de CSS, 81 de JavaScript). Pour la page seule, l'écart est faible ; il vient presque entièrement du point et du monde, que le web doit fabriquer à la main.
- La version web fonctionne dans un navigateur. Ses graines sont calculées en JavaScript avec `BigInt` et donnent les mêmes valeurs que le moteur (vérifié sur quatre valeurs de référence).
- La version HoloCode est relue par un test du moteur, qui contrôle aussi qu'aucun bloc, aucun paramètre et aucun réglage de style ne manque dans l'exemple. Tests du cœur : 29 sur 29.

![La version web de la boutique](images/2026-10-03-boutique-web.png)

**Limites, dites dans le README de l'exemple**

- La version HoloCode n'est pas encore affichée : la comparaison porte sur l'écriture, pas sur le résultat à l'écran.
- Les imports y sont en commentaire (lus par le moteur, pas appliqués). Les unités de longueur et de durée, `true` et `false` n'ont pas encore d'emploi dans le langage.
- La version web triche : le monde intérieur est chargé d'avance et caché, et ses points sont sur un cercle, pas sur une sphère.

**Erreur corrigée en route**

- Claude avait écrit `margin: 0 auto` dans le thème de la page : c'est de la disposition (centrer), que le vérificateur refuse à juste titre. Retiré de la version HoloCode, et rangé dans la partie « disposition » de la version CSS.

---

## 2026-10-03 — Les styles s'écrivent comme en CSS, avec sept règles pour servir aussi aux jeux

**Fait**

- Ordre de travail validé par Yocthan (point B) : d'abord la boutique dans les deux vues, les mesures sur téléphone en parallèle, le langage « système » plus tard.
- Yocthan a relevé que le style d'un jeu n'est pas celui d'un site, et demandé une solution cohérente dès le départ. Comparaison faite : le cœur commun est « un paquet nommé de réglages d'apparence qu'on pose sur une chose » (`.card` en CSS, un matériau dans Unity). Sept règles proposées et validées, écrites dans `ADR-017`.
- Sa condition : l'écriture reste celle du CSS de base, avec des accolades, pas un bloc `Style(...)` répété. Claude recommandait l'écriture en blocs ; Yocthan a décidé autrement, et sa raison tient : le CSS est simple à écrire, c'est son mécanisme qui pose problème. La question ouverte d'`ADR-017` est fermée.
- Moteur : le lecteur accepte `P.card(...)` et lit les styles après le bloc racine ; nouveau fichier `moteur/src/styles.rs` qui les vérifie. Tests du cœur : 28 sur 28. Les styles ne sont pas encore appliqués à l'écran.
- Suite de conformité : 22 cas (6 acceptés, 16 refusés), dont la boutique `06` et cinq refus (`E12` à `E16`).

**Choix faits par Claude, à réexaminer si besoin**

- Les blocs `Theme(...)` et `Style(...)` sont retirés : avec l'écriture CSS, le thème est le style de `Page` ou de `World`. Les garder aurait fait deux écritures pour la même chose.
- Les noms des réglages sont ceux du CSS (`font-size`, `border-radius`), mais un seul nom par réglage : `background-color` est refusé avec « écris `background` ».
- Le vérificateur ne connaît que quinze réglages, les tailles en `px` et en `%`, et une trentaine de noms de couleur. C'est plus strict que le CSS ; la liste grandira.
- Un bloc ne porte qu'un nom de style (`P.card`, pas `P.card.big`), pour ne pas avoir à départager deux styles nommés.

**Limites**

- Les accolades restent interdites dans un bloc (`ADR-015`) ; elles ne servent qu'aux styles.
- Importer un fichier de styles n'est pas encore possible : le moteur refuse tous les imports.

---

## 2026-10-03 — `Text`, `P` et `H1` : le texte comme en HTML, sans ses défauts

**Fait**

- Yocthan a demandé une comparaison avant de choisir entre `P`/`H1` et `Text`, en relevant lui-même que `Text` est le mot qui sert dans un terminal. Résultat : `Text` est le bloc de base (du texte sans rôle) ; `P` et `H1` à `H3` sont un `Text` avec un rôle. Décision écrite dans `ADR-020`, acceptée par Yocthan.
- Il a demandé pourquoi les minuscules seraient un mal. Réponse franche donnée : la majuscule n'est pas une nécessité technique, c'est une aide à la lecture (`Text(...)` le bloc, `text:` le réglage) ; le vrai défaut serait d'accepter deux écritures. Règle retenue : un bloc commence par une majuscule, un réglage par une minuscule ; `h1` est refusé avec « écris `H1` ».
- Nouvelle consigne de travail : à chaque ajout au langage, comparer d'abord les options et vérifier qu'on ne répète pas un défaut de HTML, de CSS ou de JavaScript. Écrite dans `AGENTS.md`.
- Moteur : nouveau fichier `moteur/src/blocs.rs`. Il vérifie que chaque bloc d'un fichier existe et que les titres ne sautent pas de niveau. Rien n'est encore affiché pour ces blocs. Tests du cœur : 23 sur 23.
- Suite de conformité : 16 cas (5 acceptés, 11 refusés). Le cas `01` utilise `H1` et `P` ; nouveaux cas `05`, `E09`, `E10`, `E11`.
- Question de Yocthan : le point et le pixel sont-ils la même chose, et que font Unreal et Unity ? Réponse : un pixel est sur l'écran et ne contient rien ; les moteurs 3D parlent de sommets, de particules, de voxels, de nuages de points. Deux techniques proches de sa vision, à garder en tête : le *Gaussian splatting* (un monde fait de points lumineux, tiré de photos ou de vidéos) et Nanite d'Unreal (le détail ajusté au pixel d'écran).

**Choix fait par Claude, à réexaminer si besoin**

- Yocthan avait approuvé « le vérificateur signale » un saut de niveau. Claude en a fait un refus, pas un simple avertissement : la suite de conformité ne connaît que « accepté » et « refusé », et tolérer l'erreur est justement ce qui a rendu ce défaut courant en HTML. La fiche prévoit de revenir à un avertissement si cela gêne.

**Limites**

- Le Markdown écrit dans un texte (`# Titre`) n'est pas lu par le moteur : la règle des niveaux ne porte que sur les blocs.
- `P.card(...)` n'est pas encore accepté par le lecteur ; cela attend la décision sur la place des styles.

---

## 2026-10-03 — Le point est un pixel ; la proposition de coordination de Codex est intégrée

**Fait**

- Yocthan a expliqué d'où vient le mot « point » : sa réflexion part du pixel. Une image est faite de pixels ; un monde est fait de points ; quand on zoome sur un point, il se divise en mondes. Il a laissé Claude trancher le nom selon ce que le mot désigne en 3D : **le bloc s'appelle `Point`**, défini comme « le pixel de l'Holoverse ». `Pixel` aurait heurté l'unité `px` des styles ; `Voxel` désigne un cube dans une grille. Fiche `ADR-016` complétée.
- Idée technique qui en découle, notée pour un prochain sprint : mesurer la limite de perception en pixels d'écran, et ne développer le monde d'un point que lorsqu'il occupe assez de pixels pour qu'on y voie quelque chose. Autre idée de Yocthan : partir d'une image ou d'une vidéo dont chaque pixel serait un point.
- Sur le texte, Yocthan veut l'écriture de HTML et CSS sans leurs défauts : on écrit un paragraphe, un titre, et on le stylise facilement. Claude propose des blocs `P` et `H1`, et des styles par type de bloc ou par nom à point ; à confirmer.
- Codex a répondu au prompt sur l'outillage (PR n° 19) : pas de pont direct entre IA, GitHub comme boîte aux lettres, pas de serveur MCP pour l'instant, la mesure sur téléphone par `adb` et `chrome://inspect`, et une liste priorisée des manques du projet. Yocthan a délégué la décision à Claude : la proposition est fusionnée après lecture complète, et ses parties légères sont appliquées. Un modèle de pull request et un modèle d'issue sont actifs dans `.github/`, onze étiquettes sont créées, la règle de fusion et la boîte aux lettres sont écrites dans `AGENTS.md`.

**Erreur commise**

- Claude avait daté du 4 octobre l'étape précédente et les fiches `ADR-016` à `ADR-019`, alors que tout s'est passé le 3. Les dates sont corrigées.

**Ce qui n'a pas été repris de Codex, et pourquoi**

- Le commentaire obligatoire « VALIDÉ POUR FUSION » de Yocthan sur chaque pull request : Yocthan veut aller vite et n'avoir presque rien à faire. Son accord n'est exigé que pour les pull requests des autres IA et pour celles qui changent une décision.
- Le contrôle automatique qui refuse une pull request sans les champs du modèle : gardé en exemple dans le dossier de Codex, pas activé, tant qu'on n'a pas vu le modèle servir.

---

## 2026-10-03 — Le langage passe en anglais ; quatre décisions sur la forme

**Fait**

- Yocthan a pris point par point les décisions en attente sur le langage. Les mots sont **en anglais** (contre l'avis de Codex, qui proposait le français d'abord), avec une règle née de sa remarque sur `split` : un mot que les programmeurs connaissent garde son sens. D'où `fragments` et non `split`, `leave` et non `exit`, `On` et non `When`, `brightness` et non `light`, `children` comme en Flutter. `ADR-016`.
- La forme : un `Theme`, des styles nommés avec le point du CSS (`.card`), et des réglages par bloc ; trois façons de colorer un point. `ADR-017`. La vue en profondeur : option A, seuls les points ont de la profondeur ; l'option B sera montrée pour comparer. `ADR-018`. Le texte s'écrit sans artifice, comme « Hello World » ; un `.md` ne s'importe que pour un long texte. `ADR-019`.
- Le moteur et la suite de conformité sont traduits. Le Big Bang s'écrit `Point(name: Origin, seed: 1, brightness: 1.0, fragments: 12)`. Les unités de taille deviennent `KB`, `MB`, `GB`. Le moteur refuse les anciens mots français en indiquant le mot à écrire (cas `E08`). La suite compte douze cas, et le moteur en lit trois directement dans ses tests : c'est le premier lien réel entre les deux. 19 tests.

**Reste ouvert**

- Le nom du bloc `Point` (`Point`, `Sphere` ou `Orb`) : pour un programmeur, `Point` est une coordonnée.
- La place des styles dans le fichier.

**Leçon**

- Un outil de Claude altérait les barres obliques inverses dans les commandes longues ; un test a ainsi été écrit avec un vrai saut de ligne au lieu de `\\n`. Sans gravité, mais les modifications de code passent désormais par des fichiers de script ou par l'éditeur, plus par des commandes en ligne.

---

## 2026-10-03 — La revue de Codex, et trois défauts corrigés

**Fait**

- Codex (ChatGPT) a répondu au prompt de Yocthan par une pull request (n° 15) rangée dans `proposals/GPT5.6/revue-2026-10-03/` : une revue exécutée (18 tests, compilation WebAssembly, suite de conformité, 501 856 octets Brotli mesurés), un avis sur chaque décision, des réponses aux trois questions du langage, une boutique complète en `.holo` et des sondes reproductibles. Lecture conseillée : `proposals/GPT5.6/revue-2026-10-03/README.md`.
- Trois défauts réels relevés par Codex sont corrigés dans le moteur : les graines passaient par un nombre flottant (2^53 + 1 devenait 2^53), elles sont désormais lues comme des entiers exacts ; un `import` était accepté puis ignoré en silence, il est refusé tant qu'il n'est pas appliqué ; le test qui devait figer les valeurs des graines était une tautologie, il fige maintenant de vraies valeurs. 19 tests.
- Les chiffres sont mis au propre : 19 tests et non 17 ; poids en octets exacts avec Ko = 1 000 octets (502 435 octets transférés) ; « rien n'est stocké » devient « le décor est régénérable » ; la pile de navigation coûte 16 octets par niveau ; la mémoire affichée par la page n'est que le tas JavaScript.
- Codex recommande, pour le langage : un vocabulaire français canonique en v0.1 (`import`, `module`, `pont` gardés), puis éventuellement un profil anglais par table d'alias ; un `Theme` de page plus des paramètres locaux, avec une priorité simple ; `couleur: graine` par défaut, `couleur: "#..."` ou `palette: [...]` pour l'auteur. Il propose de réordonner le travail : la boutique dans les deux vues, la navigation fiable, la mesure téléphone, puis une action avec état arbitré ; les ponts avancés et le navigateur natif ensuite. Ces choix restent à Yocthan.

**Erreurs et leçons**

- Claude a fusionné la pull request de Codex par erreur : elle portait le numéro 15 que Claude croyait être celui de sa propre pull request « pause », ouverte au même moment ; il l'a sortie du mode brouillon et fusionnée sans l'avoir lue. Aucun dégât, parce que Codex avait respecté la règle et n'avait rien écrit hors de son dossier, mais la règle « ne fusionner que ce qui a été lu et testé » a été enfreinte. Leçon : vérifier l'auteur et la branche d'une pull request avant de la fusionner, jamais seulement son numéro.
- Les trois défauts corrigés étaient dans du code que Claude avait écrit et testé : un test tautologique passe toujours. Leçon : un test doit pouvoir échouer.

---

## 2026-10-03 — Un bouton pause, et le dossier de Yocthan remis sur `main`

**Fait**

- Yocthan a demandé une pause pour le Big Bang, parce qu'il sentait la mémoire monter. Un bouton « pause » est ajouté à côté de « mesures » (PR n° 15) : le moteur ne calcule ni ne dessine plus rien jusqu'à « reprendre ». Le monde se met aussi en pause tout seul quand l'onglet est caché. Ce que la pause économise, c'est le processeur, la carte graphique et la batterie ; la mémoire du moteur, elle, est fixe (le moteur lui-même, environ 2 Mo de WebAssembly plus ce que Chrome réserve) et ne grandit pas avec la profondeur : un niveau traversé pèse 16 octets.
- Le dossier VS Code de Yocthan était resté sur une vieille branche du 21 septembre : il ne voyait ni le moteur ni les fichiers `.holo`. Il est remis sur `main`. Codex (ChatGPT) a déposé sur sa machine un dossier `TestByYou/` (une démonstration pédagogique qui extrait un bloc `holo` d'un `.md`) et une copie `revue-codex-2026-10-03/` ; ils ne sont pas dans Git.
- Explication en arbres, à la demande de Yocthan : l'arbre d'un fichier `.holo` (l'équivalent du DOM) et l'arbre du chemin, du fichier aux pixels de Chrome.

**Leçon**

- Un serveur lancé par Claude s'arrête au bout de deux heures ; il est maintenant lancé détaché de la session, et tient jusqu'au redémarrage du PC.

---

## 2026-10-03 — Le journal, AGENTS.md, et GitHub comme canal entre les IA

**Fait**

- Création de ce journal et d'`AGENTS.md` (PR n° 13), à la demande de Yocthan : un rapport tenu à chaque étape, et un point d'entrée pour toute IA branchée sur le projet.
- Rafraîchissement de mémoire sur le langage, à la demande de Yocthan : origines, décisions, correspondance HTML/CSS/JavaScript → blocs/paramètres/règles, démonstration pas à pas depuis un fichier vide, comparaison avec Dart/Flutter et avec le web classique. Trois questions restent à trancher : les mots du langage (français ou anglais), la forme (paramètres, `Theme`, ou les deux), les couleurs du Big Bang (par la graine, par l'auteur, ou les deux).
- Yocthan a demandé si Claude pouvait contrôler l'ordinateur avec un curseur visible : non depuis Claude Code ; c'est « Claude Cowork » ou « Claude in Chrome ». Pour relier les IA, il a choisi GitHub comme canal unique (commentaires de pull requests, issues) plutôt qu'un pont MCP. Gemini Code Assist est à installer sur le dépôt par Yocthan (une application GitHub ne s'installe pas en ligne de commande).

**À faire**

- Yocthan va soumettre la ligne du projet à Codex (ChatGPT) avec un prompt préparé par Claude ; la réponse arrivera dans le dépôt.
- Trancher les trois questions du langage, puis donner un sens à `Page`, `Texte`, `Bouton`, `Theme`, `Quand` dans le moteur.

---

## 2026-10-03 — Corrections après le premier essai, et publication de tout ce qui manquait

**Fait**

- Yocthan a essayé le Big Bang et a demandé trois corrections, livrées dans la PR n° 11 : les mesures sont cachées par défaut (bouton « mesures » en bas à droite, ou `?mesures=1`) ; on touche une boule pour la viser, et la rotation ne change plus une boule touchée ; la boule visée est signalée par un halo blanc qui respire.
- Les documents de fond (`VISION`, `PARADIGME`, `ARCHITECTURE`, `ROADMAP`, README) sont mis en accord avec les décisions et le sprint (PR n° 10). Le texte d'origine de ChatGPT est conservé ; les ajouts sont marqués et datés.
- Les fiches `HC-010` à `HC-012` et les transcriptions des sessions Claude sont publiées (PR n° 12). L'export est fait par `outils/exporter_sessions_claude.py`, qui ne garde que les messages de Yocthan et les réponses de Claude.
- Ce journal et `AGENTS.md` sont créés (PR n° 13).

![La boule visée, signalée par son halo ; les mesures sont cachées](images/5-boule-visee-mesures-cachees.png)

**Erreurs et leçons**

- Le moteur choisissait la cible tout seul (la boule la plus proche du centre) sans le montrer : pour Yocthan, l'entrée semblait aléatoire. Leçon : tout ce que le moteur décide à la place de l'utilisateur doit être visible, ou laissé à l'utilisateur.
- Les mesures affichées en permanence gênaient l'expérience. Un outil de sprint n'a pas sa place devant l'utilisateur.
- Le serveur de démonstration lancé par Claude s'arrête au bout de deux heures : Yocthan doit le relancer lui-même (`node outils/serveur.mjs` dans `moteur/`).

**À faire**

- Les mesures sur le Samsung Z Flip 5 (Chrome) : elles décident d'`ADR-005` et d'`ADR-010`.
- Yocthan veut maintenant parler du langage : ce que les humains écriront chaque jour, les extensions, l'écriture de sites.

---

## 2026-10-03 — Le sprint Big Bang : la première preuve exécutable

**Fait** (PR n° 9, fusionnée)

- Rust 1.99 installé sur le PC de Yocthan (profil minimal, cible WebAssembly). La connexion était lente (80 Ko/s) ; l'installation par défaut, qui téléchargeait la documentation, a été remplacée par une installation minimale.
- Un moteur en Rust (`moteur/`, 1 529 lignes), compilé en WebAssembly, qui dessine avec `wgpu` : WebGPU, ou WebGL 2 en repli. Il lit un fichier `.holo` de huit lignes au format en blocs d'`ADR-009`, le cas `02-big-bang` de la suite de conformité.
- Un point lumineux se morcelle en douze points quand on zoome ; on s'approche d'un point, on aperçoit déjà le monde qu'il contient, on y entre ; ce monde contient lui-même des points, sans fin ; en dézoomant on ressort. Rien n'est stocké : chaque monde naît de sa graine, un niveau traversé coûte 16 octets.
- 17 tests du cœur en Rust pur, qui passent aussi sur GitHub (job ajouté au flux de tests).
- Mesuré sur le PC : 1 893 Ko de moteur réel, **489 Ko transférés** (cible : moins de 2 Mo), page HTML de 1 Ko identique pour tous les mondes.

| Zoom 0 | Zoom 0,8 | Zoom 3,4 | Zoom 4,6 |
|---|---|---|---|
| ![le point entier](images/1-point-entier.png) | ![morcellement](images/2-morcellement.png) | ![plongée, monde intérieur visible](images/3-plongee-apercu-du-monde-interieur.png) | ![entré, profondeur 2](images/4-entre-profondeur-2.png) |

**Erreurs et leçons**

- `wgpu` ne retombe pas tout seul sur WebGL 2 quand WebGPU existe mais ne donne aucun adaptateur : le repli a dû être écrit à la main, en recréant la zone de dessin (une zone qui a reçu un contexte WebGPU n'en accepte plus un autre).
- Deux tests du cœur ont échoué au premier passage : avance octet par octet dans un texte accentué, et points pas exactement sur la sphère. Corrigés avant toute publication.
- Les images par seconde mesurées sur le PC sans carte graphique n'ont aucun sens ; elles n'ont pas été reportées. La seule mesure qui compte est celle du téléphone, et elle n'est pas faite.

---

## 2026-10-02 — Tout fusionner, puis ne garder que ce qui marche

**Fait**

- Sur ordre de Yocthan, les cinq pull requests en attente depuis le 21 septembre ont été fusionnées dans `main` : les trois prototypes, les décisions, les tests automatiques et la suite de conformité.
- Les tests automatiques ont alors montré ce que les README ne disaient pas : ChatGPT 8 sur 8, Claude 27 sur 27, Gemini 3 sur 4. `main` était rouge.
- Yocthan a précisé la règle : on ne fusionne que ce qui marche, et on revoit le reste. Gemini a reçu un fichier de consignes avec les résultats réels et la revue ; il a rendu une version corrigée (PR n° 7) : 6 tests sur 8, cinq défauts sur huit réellement corrigés. Les deux échecs venaient d'une seule ligne, qui empêchait d'entrer dans un nœud sans enfants. Sur ordre de Yocthan, Claude a corrigé cette ligne dans un commit à son nom : 8 sur 8, `main` au vert.
- Le README de Gemini a été corrigé là où la mesure le contredisait (PR n° 8) : la mémoire comptée n'est pas constante, elle croît de 64 Ko par niveau descendu.

**Erreurs et leçons**

- Claude a fusionné la proposition de Gemini alors qu'un test échouait, parce que l'ordre « tout fusionner » est arrivé avant la règle « seulement ce qui marche ». Une pull request de retrait a été préparée puis fermée sans fusion, la correction ayant suffi.
- Gemini a deux fois déroulé ses tests « à la main » en suivant son intention plutôt que son code. Leçon pour toutes les IA : un résultat de test ne s'annonce pas, il s'exécute.
- Le copier-coller depuis l'interface de Gemini abîme le code (indentation, `__init__` en gras) ; la demande de mettre chaque fichier dans un bloc de code a réglé le problème.

---

## 2026-09-21 — Les décisions, les trois prototypes et la revue de ChatGPT

**Fait**

- Claude a lu le dépôt créé par ChatGPT et donné son avis franc : le paradigme holoscénique n'a pas de primitive nouvelle (ECS, Datalog, règles de production, F#, Inform 7, Verse), et sa valeur possible est la synthèse ; l'originalité du projet est dans la vision de Yocthan (le point qui contient des mondes, le zoom, le web qui devient métavers, le budget de 1 Go).
- Trois prototypes en Python, un par IA : ChatGPT (noyau minimal), Claude (lois, capacités, vérificateur, journal causal), Gemini (nœud fractal, zoom, budget mémoire). Comparés par Claude, juge et partie ; ChatGPT a relevé qu'ils ne testent pas le même problème.
- Yocthan a pris ses premières décisions techniques, inscrites dans `ADR-007` à `ADR-015` : le métavers est une mise à jour du web avec deux vues ; le fichier `.holo` est une vraie source ; des blocs nommés par leur sens, texte Markdown dans les blocs ; moteur en Rust ; ponts JavaScript et CSS seulement en première version ; l'IA crée mais le monde se lit sans elle ; tout changement d'état passe par un arbitre. Il a aussi tranché les propositions de ChatGPT (`ADR-003` à `ADR-005` acceptées, `ADR-006` en proposition).
- ChatGPT a relu l'ensemble (note globale 5,5 sur 10, preuve sur téléphone 0 sur 10) : trop de décisions acceptées avant mesure (quatre sont passées en `EXPÉRIMENTATION`), `ADR-014` trop absolue (reformulée), dossiers non uniformes (uniformisés), pas de tests automatiques (ajoutés), pas de banc commun (suite de conformité créée).

**Erreurs et leçons**

- Claude a d'abord répondu « non » à la question « y aura-t-il des appels ? » avant de se corriger : il y en a, de deux sortes, et une troisième est interdite. Les absolus ne tiennent pas.
- Yocthan a observé que l'holoscénique ressemblait à l'objet : c'est vrai, et la description honnête l'a remplacé dans les documents. Ne pas revendiquer une révolution que le premier programmeur venu démonterait.
- « La documentation court plus vite que le moteur » (ChatGPT) : les trois IA et Yocthan ont fait de la preuve sur téléphone la priorité.

---

## 2026-09-20 — Le cadrage de la vision

Yocthan expose à Claude sa vision (`HC-011`) : remodeler l'usage du web en monde explorable, par sprints de 24 heures, horizon 2030. Le Big Bang : un point lumineux qui se morcelle en points, chacun contenant un monde ; la sphère est aussi le premier personnage. Le premier test doit tenir dans 1 Go et tourner sur n'importe quel téléphone actuel. Genie 3 n'est accessible qu'en inspiration. Yocthan veut un langage entièrement nouveau, verbeux mais simple, dans l'esprit de Dart et Flutter. Claude propose la génération par graine et objecte que battre C, Rust et Zig comme langage généraliste est hors de portée d'une personne seule.

---

## 2026-09-19 — Le faux départ

Claude a reçu le brief de Yocthan et a construit, sans en discuter, une première démonstration (sphère en WebGL, JavaScript, 24 Ko), en commençant même à nommer un langage. Yocthan l'a arrêté : rien n'avait été discuté. Tout a été effacé. Règle née ce jour-là, et depuis toujours respectée : **on discute d'abord ; aucune création de fichier, aucune décision technique, aucun nom sans son feu vert.** Détail dans `HC-010`.
