# Essai au lecteur d'écran : le protocole

Écrit le 2026-10-06 par Claude (`ADR-055`). Les outils automatiques (axe-core, le contrôle de contraste du moteur) ne trouvent plus aucun défaut dans les leçons et les sites d'exemple, en thème clair et sombre, sur PC et en largeur de téléphone. Mais ils ne savent pas **ce qu'entend vraiment une personne aveugle**. Cet essai, seul un humain peut le faire. Il dure environ 30 minutes.

## Préparer

- **Sur le Galaxy Z Flip** : Paramètres → Accessibilité → TalkBack → activer. Gestes : glisser un doigt vers la droite pour passer à l'élément suivant, vers la gauche pour revenir ; toucher deux fois pour activer ; glisser deux doigts pour faire défiler. Pour arrêter TalkBack : appuyer en même temps sur les deux touches de volume pendant trois secondes (si le raccourci est activé), sinon par les Paramètres.
- **Sur le PC** (facultatif) : NVDA, gratuit (nvaccess.org). Touche `Tab` pour aller de bouton en bouton, flèche du bas pour lire, `H` pour sauter de titre en titre, `D` pour sauter de repère en repère.
- Le serveur d'essai tourne sur le PC ; le téléphone ouvre l'adresse affichée (`http://192.168.x.x:8080`).

## Les pages à essayer, et ce qu'il faut entendre

| Page | Ce qu'on fait | Ce qu'on doit entendre | Résultat |
|---|---|---|---|
| Leçon 1 (`01-page.holo`) | lire du haut en bas | le titre de l'onglet, puis « Titre de niveau 1 », puis le texte | |
| Leçon 35 (`35-reperes.holo`) | sauter de repère en repère | « en-tête », « navigation », « principal », « pied de page » | |
| Leçon 3 (`03-image-liste-lien.holo`) | aller sur l'image et la liste | le texte de remplacement de l'image ; « liste, 3 éléments » | |
| Leçon 16 (`16-saisie.holo`) | aller sur le champ, écrire | le nom du champ (son étiquette) avant de pouvoir écrire ; la case à cocher et son état | |
| Leçon 63 (`63-fenetre.holo`) | ouvrir la fenêtre, puis la fermer | « boîte de dialogue » ; le bouton de fermeture ; le retour sur le bouton qui l'a ouverte | |
| Leçon 64 (`64-formulaire.holo`) | remplir et envoyer | chaque champ et son étiquette ; le message « envoyé » ou « échec » | |
| Leçon 68 (`68-liste-qui-change.holo`) | ajouter une tâche, en retirer une | la nouvelle ligne ; le nombre de tâches qui change | |
| Leçon 70 (`70-composants.holo`) | toucher « Ajouter » sur une carte | le titre de la carte ; le total qui change | |
| Leçon 71 (`71-liste-a-champs.holo`) | parcourir le catalogue, ajouter | chaque article : titre, prix, note ; le nombre d'articles | |
| Leçon 72 (`72-place-et-theme.holo`) | aller sur le champ de recherche | « Un tableau, champ d'édition » | |
| Site de référence, `catalogue.holo` puis `panier.holo` | ajouter un tableau, aller au panier | le menu, les articles, le panier qui se remplit | |

Pour chaque ligne : **ok**, ou ce qui manquait (un bouton sans nom, une valeur qui change sans être annoncée, un ordre de lecture étrange, un piège où l'on ne peut plus sortir).

## Ce qu'on fait des résultats

Coller le tableau rempli dans une discussion ou dans `docs/05-discussions/reponses/`. Chaque problème devient une correction du moteur, pas de la leçon : si un bouton n'a pas de nom, c'est le moteur qui doit refuser un bouton sans texte, ou lui en donner un.

## Ce qui est déjà vérifié par les outils

- `node moteur/outils/accessibilite.mjs exemples/lecons` (axe-core 4.14, dans Chrome) : 72 pages, 0 défaut, en thème clair et sombre, à 1000 et à 390 pixels de large ; idem pour `site-reference`, `site`, `boutique-comparee`, `jeu`, `maison`, `zoom`.
- Le moteur refuse un style dont le texte et le fond, écrits ensemble, sont trop peu contrastés (4,5 pour 1, ou 3 pour 1 pour un grand texte).
- Le moteur exige déjà le texte de remplacement des images (`alt`), fabrique de vrais repères (`header`, `nav`, `main`, `footer`), de vrais titres, de vrais champs avec leur étiquette, et le survol marche aussi au clavier.
