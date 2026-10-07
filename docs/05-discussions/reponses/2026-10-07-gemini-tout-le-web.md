# Réponse de Gemini — tout ce que le web sait faire, avant le métavers (2026-10-07)

Réponse au prompt [`../prompts/2026-10-07-codex-gemini-tout-le-web.md`](../prompts/2026-10-07-codex-gemini-tout-le-web.md), collée par Yocthan le 2026-10-07. Gemini n'a pas accès au dépôt. Texte gardé tel quel ; la lecture de Claude est à la fin.

---

**1. Peut-on vraiment atteindre « tout le web » sans code libre ?**

Oui, mais seulement si « tout le web » désigne les sites, boutiques, formulaires, outils de gestion et espaces partagés courants.

En revanche, les applications au calcul imprévisible (un tableur complet façon Excel, un éditeur d'image avec calques ou un jeu d'action physique complexe) ne peuvent pas se limiter à des mots déclaratifs : elles exigent obligatoirement le relais d'un module WebAssembly enfermé (un petit bloc de code compilé, isolé du reste de l'ordinateur comme dans une boîte étanche) pour exécuter des calculs sur mesure sans mettre la machine en danger.

**2. Les lots ordonnés pour payer la dette du web**

Voici le découpage pragmatique en 8 lots pour solder le web fonctionnel avant de basculer sur la 3D :

Lot 1 — Le traitement des données et du texte (3 séances)
- Ce qu'on pourra faire : Un catalogue avec recherche instantanée, tri (par prix ou alphabétique), pagination et prix avec centimes réels.
- Moyen : Fonctions pures dans le moteur Rust (filtrer, chercher, tronquer, trier) et nombres décimaux à virgule fixe (deux chiffres après la virgule, comme pour la monnaie réelle).

Lot 2 — Les formulaires sûrs et accessibles (2 séances)
- Ce qu'on pourra faire : Une inscription ou commande avec validation immédiate (e-mail valide, mot de passe assez long), affichage des erreurs sous chaque champ, envoi par la touche Entrée et blocage du double-clic accidentel.
- Moyen : Mots du langage (required, type: email, min: 8) et logique interne au moteur pour piloter l'accessibilité vocale.

Lot 3 — Les vrais écrans : ordinateur et téléphone (2 séances)
- Ce qu'on pourra faire : Un site qui s'élargit proprement sur grand écran (au-delà de 640 pixels) et réorganise ses blocs sans empilement forcé.
- Moyen : Mots du langage : des règles d'écran simples (wide: pour grand écran, phone: pour mobile) et le réglage de ce qui dépasse (overflow: scroll ou hidden).

Lot 4 — Le serveur local et les valeurs partagées : holo serve (4 séances)
- Ce qu'on pourra faire : Une boutique où l'administrateur change un stock sur son PC et où tous les visiteurs connectés au Wi-Fi voient le stock baisser en direct sans recharger la page.
- Moyen : Serveur Rust autonome avec base de données SQLite locale intégrée, et mots de liaison (shared: [stock], mine: [panier]).

Lot 5 — Les comptes et la sécurité « chez soi d'abord » (4 séances)
- Ce qu'on pourra faire : Créer un compte, se connecter avec mot de passe chiffré et code à 6 chiffres, sans aucun service extérieur. L'administrateur gère tout depuis son terminal.
- Moyen : Serveur Rust avec SQLite, mot de passe chiffré via l'algorithme Argon2, et gestion en ligne de commande.

Lot 6 — Les détails indispensables du HTML (2 séances)
- Ce qu'on pourra faire : Ouvrir des liens dans un nouvel onglet, télécharger un fichier d'un clic, afficher les sous-titres d'une vidéo et insérer des blocs d'à-côté (Aside).
- Moyen : Mots du langage et balisage HTML conforme par le moteur.

Lot 7 — L'élargissement des modules WebAssembly (4 séances)
- Ce qu'on pourra faire : Faire tourner des calculs sur mesure (manipulation d'un tableau de données, formatage monétaire complexe, algorithmes personnalisés) en toute sécurité.
- Moyen : Modules WebAssembly enfermés capables d'échanger des textes et des listes de données, et non plus de simples nombres isolés.

Lot 8 — Le dessin vectoriel et le hors-ligne (3 séances)
- Ce qu'on pourra faire : Dessiner des icônes ou graphiques vectoriels nets à toute échelle, et consulter les pages du site même sans connexion réseau.
- Moyen : Bloc vectoriel déclaré dans le langage (Svg / Shape) et fichier de mise en cache locale généré par le moteur.

Total estimé : environ 24 séances de travail.

**3. Réponses aux autres questions**

Quelles dettes payer avant la 3D, lesquelles attendre, lesquelles refuser ?
- À payer avant la 3D : Tout ce qui permet d'afficher, trier, chercher des articles, commander et gérer un compte (Lots 1 à 5). Sans cela, un magasin 3D ne sera qu'une coquille vide sans stock ni paiement.
- À faire attendre (après la première chaise 3D) : Les filtres graphiques sophistiqués (flou d'arrière-plan, découpes géométriques complexes), la géolocalisation et l'accès à la caméra.
- À refuser franchement : Le dessin libre impératif direct (façon Canvas2D où l'on trace trait par trait en JavaScript). C'est le retour du code spaghetti incontrôlable.

Le serveur et les comptes : le plan est-il bon ?
Le plan est excellent. Utiliser un seul arbitre Rust exécuté à la fois dans le navigateur et sur le serveur garantit que personne ne peut tricher ou inventer un état faux. L'utilisation d'un fichier SQLite local (une base de données complète qui tient dans un simple fichier sur le disque) respecte parfaitement la règle « chez soi d'abord ». C'est une fondation solide pour le futur métavers, où chaque monde possédera son propre serveur local ou distant.

Les larges capacités de JavaScript (dessin, dates, matériel)
- Textes et dates : Doivent être intégrés sous forme de filtres déclaratifs purs dans le moteur (ex. Date.format("DD/MM/YYYY")).
- Dessin : Remplacer le dessin par script par des blocs vectoriels déclaratifs (formes géométriques, tracés fermés).
- Matériel (caméra, position) : Utiliser des blocs d'autorisation explicites avec signaux (Permission(camera), On(Allow, effect: ...)).
- Modules WebAssembly : Il faut impérativement leur permettre de recevoir des chaînes de texte et des listes d'octets sérialisées (formats de données compacts), tout en leur interdisant tout accès direct au système de fichiers et au réseau.

La mise en page sans les défauts du CSS
Il ne faut pas réintroduire les requêtes média arbitraires du CSS (qui éclatent les règles de style en morceaux). La bonne solution consiste à déclarer des paliers clairs sur les blocs eux-mêmes :
- Page(maxWidth: 1200px) pour centrer le contenu sur un grand moniteur d'ordinateur.
- Des attributs conditionnels simples sur les blocs de disposition, par exemple Row(phone: Column) : le bloc s'affiche en ligne sur un ordinateur, mais empile ses éléments en colonne dès que l'écran est étroit.

**4. Les risques qui menacent ce qui marche**
- L'illusion du « zéro code » face aux règles métier : Si une boutique demande des règles de calcul complexes (ex. « -10% le mardi pour les comptes créés il y a plus d'un an, sauf sur les articles en promotion »), le système déclaratif risque de devenir lourd et illisible. Le passage par un module WebAssembly isolé doit être la soupape de sécurité pour ces calculs spécifiques.
- La divergence entre l'affichage à plat et l'affichage en points : Plus le HTML et le CSS générés s'enrichissent de blocs élaborés (tableaux, formulaires avec messages d'erreur flottants), plus la transposition automatique de ces éléments en nuage de points volumétrique devient ardue à maintenir de façon cohérente.
- Le coût mémoire de la synchronisation en direct : Un serveur WebSocket local qui gère des dizaines de connexions tout en mémorisant l'historique complet de chaque geste peut saturer la mémoire vive d'un petit ordinateur si l'on ne nettoie pas régulièrement les anciens états (compactage de journal).

**5. Ce qu'il faut faire cette semaine (en 5 lignes)**
- Implémenter les nombres à virgule fixe (prix en centimes avec affichage décimal automatique) et le filtrage élémentaire d'une liste (recherche de sous-chaîne dans un texte).
- Ajouter la validation basique des champs de formulaires (champ obligatoire et format d'e-mail avec message d'erreur d'accessibilité associé).
- Définir un paramètre maxWidth sur la Page pour briser la limitation artificielle des 640 pixels sur grand écran d'ordinateur.

---

## Lecture de Claude

- **Le bon prompt.** Gemini répond au prompt « tout le web » (celui qui a remplacé « fin du web » le même jour), pas au premier : c'est le bon.
- **Vérifié dans le moteur, vrai** : le contenu de la page est limité à 640 px de large (`max-width:640px` sur `main`, `header`, `footer`) ; il n'y a ni recherche, ni tri, ni pagination dans une liste ; les valeurs sont des nombres entiers (`{n:cents}` n'est qu'un affichage) ; les champs n'ont ni `required`, ni `type: email`, ni longueur minimale ; ni `Aside`, ni lien vers un nouvel onglet, ni téléchargement, ni sous-titres ; un module n'échange qu'un nombre ; ni SVG dans la page, ni hors-ligne.
- **D'accord** : le serveur et les comptes comme proposés (un seul arbitre, SQLite, Argon2, code à 6 chiffres, gestion au terminal) ; refuser le dessin trait par trait ; des paliers sur les blocs plutôt que des requêtes média libres (`phone:` existe déjà comme état de style).
- **À discuter** : `Row(phone: Column)` mélange deux blocs dans un réglage ; une écriture comme `Row(phone: stack)` ou un style `phone: { direction: column }` resterait dans la grammaire actuelle. `Date.format("DD/MM/YYYY")` est une écriture de fonction : HoloCode a déjà les formats dans les textes (`{day}/{month:00}/{year}`, `ADR-043`).
- **Il manque** : un ordre de grandeur réaliste. 24 séances avant la 3D, c'est beaucoup ; les lots 1 à 3 (environ 7 séances) sont petits et sans risque, les lots 4 et 5 touchent à la sécurité.
- **Codex** n'a pas encore répondu.
