# Prompt pour Codex (GPT) et Gemini — tout ce que le web sait faire, avant le métavers

- Écrit par Claude le 2026-10-07, à la demande de Yocthan : « c'est mieux que tu crées le bon prompt que je vais passer à Codex et à Gemini ».
- Il remplace le prompt du même jour, `2026-10-07-codex-gemini-fin-du-web.md`, qui proposait de déclarer le web fini avec ce qui existe : cela contredisait le but que Yocthan a redit le même jour (« le code doit d'abord faire tout ce que les HTML CSS JavaScript savent faire »).
- Le même texte pour les deux. Gemini n'a pas accès au dépôt : tout ce qu'il faut est dans le texte. Les réponses iront dans `docs/05-discussions/reponses/`.

---

Bonjour. Je m'appelle Yocthan. Je construis HoloCode, un langage pour construire des sites web et un métavers, de A à Z, avec l'aide de Claude (Anthropic), de Codex (OpenAI) et de Gemini (Google). J'ai besoin de ton avis franc. Je ne suis pas programmeur web : **réponds en français simple, et explique chaque mot technique la première fois que tu l'emploies.**

## 1. Mon but, le plus important

> « Le code doit d'abord faire tout ce que les HTML, CSS, JavaScript savent faire, et ensuite faire le métavers, et de la meilleure manière. » « Il faut d'abord rendre le web actuel viable. »

Notre règle, décidée le 6 octobre : **HoloCode refuse une mécanique, jamais une capacité.** Il peut refuser une façon de faire du web (le JavaScript libre, la balise `div`, la cascade du CSS), mais jamais une chose que le web permet de faire. Un besoin du web sans réponse est une **dette**, à payer avant de dire que le web est fini.

Et une deuxième règle, **chez soi d'abord** :

> « Je ne suis pas obligé d'aller chez Google ou bien chez un prestataire quelconque pour une version d'un site qui tourne d'abord dans mon ordinateur. Mais par après, c'est là que j'y vais […] mais pas dès le départ. »

Comme avec Django : sa base de données, ses comptes et sa double authentification, chez soi, sur son ordinateur. Les prestataires viennent plus tard, au choix.

**Ce que je te demande : m'aider à payer toutes les dettes du web, dans le bon ordre, avant la 3D.**

## 2. HoloCode en quelques lignes

- On écrit un fichier `.holo` : des blocs (`Page`, `Text`, `Button`, `Row`…), des styles comme en CSS, des règles (`On(Add.tap, effect: cart.add(1))`), des valeurs (`State(cart: 0)`).
- **Aucun code libre** : l'auteur n'écrit jamais de JavaScript. Un moteur écrit en Rust, compilé en WebAssembly, lit le fichier et fabrique du vrai HTML et du vrai CSS. Ce qui est mal écrit est refusé, avec le bon mot.
- Le cœur du moteur est un **arbitre** : une fonction qui prend un état et un geste (« le bouton Add est touché ») et rend le nouvel état. Un même fichier donne toujours le même résultat.
- Une page peut aussi devenir un monde de points dans lequel on zoome. La 3D viendra après le web ; son plan est prêt (une chaise réaliste d'abord, environ 15 séances).
- Aujourd'hui : 333 mots du langage, tous décidés ; 81 leçons ; 124 tests. Mesuré sur un PC (pas sur un téléphone) : une page simple pèse 8 Ko à l'ouverture ; quand elle a besoin du moteur, 193 Ko ; en vue points, 846 Ko.

## 3. Ce que le web de HoloCode sait déjà faire

- Pages, titres, textes, images (avec une version plus légère pour téléphone), sons, vidéos, tableaux, fenêtres par-dessus la page, plis qui s'ouvrent ; la langue, la description et l'image de partage de la page.
- La mise en page : `Row`, `Column`, `Grid`, un bloc qui prend la place restante (`grow`), une superposition (`Stack`) ; des thèmes partagés, le mode sombre, le téléphone ; les tailles qui suivent la taille du texte choisie par le visiteur.
- Des composants : paramètres, valeurs par défaut, signaux branchés par la page, un emplacement pour du contenu.
- Des listes : de textes ou d'éléments à champs, remplies par un geste ou par un fichier JSON du serveur, avec une condition par ligne.
- Des règles : le toucher, le survol (souris, clavier, doigt), toutes les touches utiles du clavier, le temps, un hasard rejouable, glisser sur un plateau, des mouvements et des apparitions quand on descend dans la page.
- Des formulaires qui envoient leurs valeurs et un fichier au serveur.
- Du code compilé (WebAssembly) enfermé, borné en temps et en mémoire : aujourd'hui, il reçoit un nombre et rend un nombre.
- L'accessibilité : les étiquettes sont obligatoires, le contraste trop faible est refusé, un audit automatique ne trouve aucun défaut sur les leçons, et même la vue points se lit au lecteur d'écran.
- Des outils : un éditeur dans le navigateur, une extension VS Code, un panneau des valeurs, des essais écrits.

**Tout tourne sur le PC de l'auteur.** Un petit serveur d'essai en Node.js sert les pages et range les messages des formulaires dans un dossier.

## 4. Ce qui manque encore : la liste des dettes

Relevé de Claude du 7 octobre, vérifié dans le code. Son estimation de la couverture : 77 % du web (HTML 84 %, CSS 80 %, JavaScript 59 %).

**A. Les données et le calcul**
- Chercher un mot dans une liste ; filtrer selon un choix du visiteur ; trier ; afficher page par page. Aujourd'hui, un texte ne se compare qu'au vide.
- Les nombres à virgule (seulement des nombres entiers : un prix s'écrit en centimes) ; les dates (pas de « dans trois jours ») ; travailler un texte (majuscules, longueur, découper).
- Des identifiants stables choisis par l'auteur pour les éléments d'une liste.
- Une liste reçue du serveur ne peut dire ni « chargement… » ni « échec », et la page fabriquée par le serveur ne la contient pas (un robot de recherche ne voit pas le catalogue).

**B. Les formulaires**
- La vérification des champs (obligatoire, adresse e-mail, longueur), les erreurs écrites à côté du champ et annoncées au lecteur d'écran, envoyer avec la touche Entrée, ne jamais envoyer deux fois, un délai si le serveur ne répond pas.

**C. Le serveur et le réseau**
- Des **valeurs partagées** (la même pour tous les visiteurs : Ada réserve un tableau, Bob voit qu'il est réservé), des **comptes**, le **direct** (voir les autres changer la page sans recharger), un **vrai serveur**.
- Une adresse par personne ou par objet (`/profil/123`) : aujourd'hui, une page égale un fichier.
- Ce qui se garde le temps d'une visite (cookies, sessionStorage) ; le site qui marche hors ligne ; le paiement (plus tard, chez un prestataire).

**D. La mise en page et le dessin des pages**
- Une vraie page d'ordinateur et une vraie page de téléphone : la page fait au plus 640 pixels de large, et il n'y a qu'un seul seuil, celui de l'écran. Pas de variante selon la place que reçoit un bloc (comme les requêtes de conteneur du CSS).
- Ce qui dépasse d'un bloc (`overflow`), la forme du curseur, les proportions (`aspect-ratio`), le flou et les filtres de couleur, les découpes (`clip-path`), les graisses intermédiaires d'une police, le texte justifié.

**E. Le HTML qui manque**
- `aside` ; `abbr`, `time`, `address` ; les listes de définitions ; ouvrir un lien dans un nouvel onglet, télécharger un fichier ; `fieldset`, `legend`, des suggestions dans un champ ; une navigation nommée qui dit « vous êtes ici ».

**F. Les médias**
- Les sous-titres d'une vidéo, son image d'attente ; les images chargées seulement quand on approche ; les sons téléchargés seulement quand on va les jouer.

**G. Le dessin libre et l'appareil**
- Des dessins vectoriels dans la page (SVG), le dessin libre (Canvas 2D), des particules, la position du pointeur.
- La géolocalisation, la caméra, la vibration, le presse-papiers, le bouton « partager ».

## 5. Ce qui est déjà proposé pour le serveur (par Claude)

**Un seul moteur, des deux côtés.** `holo serve`, un serveur écrit en Rust, fait tourner le même arbitre que la page. La page envoie des gestes, pas des valeurs ; le serveur rejoue chaque geste sur sa propre copie et ne croit jamais ce que dit le navigateur. L'auteur n'écrit pas de serveur : il dit, dans sa page, ce qui est gardé et partagé.

| Écriture | Où vit la valeur | Qui la voit |
|---|---|---|
| `State(likes: 0)` | dans l'onglet du visiteur | lui seul |
| `keep: [likes]` (existe) | dans son navigateur, d'une visite à l'autre | lui seul, sur cet appareil |
| `shared: [likes]` (proposé) | sur le serveur | tout le monde, la même valeur |
| `mine: [cart]` (proposé) | sur le serveur, une par compte | chacun la sienne, sur tous ses appareils |

Quatre étapes, estimées à 10 à 12 séances : `holo serve` ; les valeurs partagées (d'abord sur le PC et le Wi-Fi de la maison, changées seulement par un geste, chaque geste numéroté et appliqué une fois) ; les comptes ; le direct à plusieurs (WebSocket).

Les comptes, dans la base de l'auteur (un fichier SQLite) : mot de passe chiffré (Argon2) avec un code à 6 chiffres (TOTP) d'abord, puis les clés d'accès (passkeys) ; un lien de connexion affiché dans le terminal de l'auteur sur son PC ; « Se connecter avec Google » seulement plus tard, en option. Un mot de passe oublié se remet par une commande de l'auteur (`holo account reset ada`), comme `manage.py changepassword` de Django.

Ce que vous aviez dit le 4 octobre, sur le jeu à plusieurs : **Codex** : le serveur doit posséder vraiment l'état ; ne jamais accepter l'état envoyé par le navigateur ; un numéro unique par geste, dans un seul ordre ; un bouton caché n'est pas une permission ; le temps et le hasard décidés par le serveur. **Gemini** : deux modèles, le réflecteur (le serveur numérote les gestes, chaque navigateur calcule, comme Croquet) ou le serveur qui fait autorité (comme Roblox ou Colyseus) ; des coordonnées logiques fixes ; le direct par WebSocket en Rust. La proposition choisit le serveur qui fait autorité.

## 6. Comment HoloCode peut donner une capacité sans code libre

Il a quatre moyens :

1. **un mot du langage** : un bloc, un paramètre, une règle (comme `Form`, `Repeat`, `grow`) ;
2. **une fonction pure** : un calcul sans effet de bord, décidé par le moteur (trier, chercher, formater une date) ;
3. **un module WebAssembly enfermé** : du code compilé, sans réseau ni page, borné en temps et en mémoire (aujourd'hui un nombre contre un nombre) ;
4. **le serveur de l'auteur** : ce que le navigateur ne doit pas décider seul (les comptes, ce qui est partagé, les droits).

## 7. Mes questions

1. **Peut-on vraiment atteindre « tout le web » de cette façon, sans code libre ?** Oui ou non, et à quelle condition. S'il y a des capacités que tu juges impossibles ou dangereuses sans code libre, nomme-les.
2. **Quelles dettes faut-il payer avant la 3D**, lesquelles peuvent attendre, et lesquelles refuser franchement ? Pour chacune, pourquoi.
3. **Dans quel ordre ?** Propose des lots (de 5 à 10). Pour chaque lot : ce qu'on pourra faire à la fin (par exemple « un catalogue où l'on cherche et trie »), le moyen (mot, fonction pure, module, serveur), et une estimation en séances.
4. **Le serveur et les comptes** : le plan est-il bon ? Respecte-t-il « chez soi d'abord » ? Faut-il le faire tôt (puisque le métavers à plusieurs en aura besoin) ou après les autres dettes ?
5. **Les plus larges capacités de JavaScript** : le dessin libre, le travail sur les textes et les dates, l'appareil (position, caméra), le hors ligne. Comment les donner sans code libre ? Faut-il élargir les modules enfermés (des textes, des listes, une zone de dessin), et avec quels garde-fous ?
6. **La mise en page** : comment avoir une vraie page d'ordinateur et une vraie page de téléphone, sans répéter les défauts du CSS ?
7. **Les risques** : la sécurité, le poids, l'accessibilité, et la règle « un même fichier donne toujours le même résultat ». Qu'est-ce qui pourrait casser ce qui marche ?
8. **Ce que nous avons oublié** : une capacité du web absente de la liste, une erreur de chiffre, un risque.

## 8. Comment répondre

Dans cet ordre, en français simple :

1. ta réponse à la question 1, en deux phrases ;
2. ta liste de lots, classée, avec leurs estimations ;
3. une réponse courte à chacune des autres questions ;
4. les risques que tu vois ;
5. ce que tu ferais en premier, cette semaine, en cinq lignes au plus.

Sois franc : si une idée de Claude est mauvaise, ou si notre but te paraît mal posé, dis-le, et dis ce que tu ferais à la place.

**Si tu as accès au dépôt GitHub `yocthanmabeka/Metaverse` (Codex)** : lis aussi `AGENTS.md`, `docs/01-holocode/TABLEAU-WEB.md` (le grand tableau face au web), `docs/02-gouvernance/adr/ADR-035-une-mecanique-jamais-une-capacite.md`, `proposals/Claude/serveur-et-comptes-2026-10-07.md`, et ta propre revue `proposals/GPT5.6/web-assez-utilisable-2026-10-07/README.md`. Si le rapport de Claude sur tes dix pistes est arrivé (`proposals/Claude/exploration-web-complet-2026-10/`), lis-le aussi. Vérifie ce que tu peux en lançant le code. Ne change rien dans `main` : mets ta réponse dans une pull request, sous `proposals/GPT5.6/`.
