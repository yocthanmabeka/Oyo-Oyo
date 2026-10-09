# Prompt : huit fonctions du web que HoloCode limite exprès. Faut-il les garder limitées ?

- Écrit par la session Claude du PC, le 2026-10-09, à la demande de Yocthan.
- Pour Codex (ChatGPT), Gemini et toute autre IA.
- À donner tel quel. Les réponses vont dans `proposals/<ton nom>/contraintes-2026-10-09/README.md`, par une pull request si tu as accès au dépôt `yocthanmabeka/Oyo-Oyo` ; sinon, à Yocthan, qui les y rangera.

---

Bonjour. Je suis Yocthan, je ne suis pas programmeur. Je construis HoloCode, un langage pour écrire des sites web, et plus tard des mondes en 3D.

Claude, qui programme avec moi, a classé huit fonctions du web comme « limitées exprès » : HoloCode n'en fait qu'une partie, volontairement. Moi, je ne suis pas d'accord sur plusieurs d'entre elles, et je veux un avis indépendant avant de décider.

**Ce que je te demande**, pour chacune des huit :

1. **Trouve les contraintes réelles** : sécurité, vie privée, lois (RGPD, paiement), coût, poids sur un téléphone, accessibilité, compatibilité des navigateurs, téléphone et ordinateur. Cherche sur Internet et **donne tes sources** (lien et date). N'invente jamais une source.
2. **S'il n'y a pas de contrainte, crée-en** : imagine ce qui pourrait mal tourner, et écris clairement « contrainte imaginée, sans source ».
3. **Donne ton verdict** : garder limité, ouvrir entièrement, ou ouvrir sous conditions. Si tu ouvres, propose une forme courte en HoloCode, qui respecte les règles ci-dessous.
4. **Dis ce que tu n'as pas pu vérifier.**

Réponds en français, simplement, une partie par fonction.

## Les règles de HoloCode, qu'une proposition doit respecter

- **L'auteur n'écrit jamais de HTML, de CSS ni de JavaScript.** Il écrit des blocs (`Page`, `Button`, `Input`…), et le moteur, écrit en Rust, fabrique la page.
- **Aucun code libre dans une page, et aucun pont vers JavaScript.** Le calcul d'un auteur passe par des modules WebAssembly « enfermés » : mémoire plafonnée, temps limité, pas d'accès au réseau ni à la page, des données vérifiées à l'entrée et à la sortie.
- **Chez soi d'abord** : aucun prestataire (Google, Apple, un hébergeur, un service de paiement…) n'est jamais *obligatoire* pour qu'un site tourne sur le PC de son auteur. Un prestataire peut être *permis*, s'il est choisi par l'auteur et qu'il est facultatif.
- **Le serveur de l'auteur** (`holo serve`, en Rust, avec SQLite) fabrique les pages, garde les comptes (mot de passe Argon2id, code à 6 chiffres, clés d'accès WebAuthn) et marche aussi sans JavaScript.
- **Parité** : tout ce qui marche sur un ordinateur marche sur un téléphone, et l'inverse (souris et clavier, doigt, lecteur d'écran).
- **Accessibilité vérifiée** par le moteur : les noms obligatoires, le contraste, le lecteur d'écran.
- **Le déterminisme** : un même fichier donne la même page.

## Les huit fonctions

### 1. Une page dans la page (`iframe`, `embed`, `object`)

- **Aujourd'hui** : `Point(inside: "x.holo")` entre dans un autre fichier HoloCode. Mettre la page d'un *autre site* dans la sienne est impossible.
- **Pourquoi Claude l'a limité** : la sécurité (de faux boutons posés par-dessus, le « clickjacking »), le pistage du visiteur par un autre site (cookies tiers), un contenu que l'auteur ne contrôle pas, le poids sur un téléphone.
- **Mon objection** : la plupart des réseaux sociaux marchent avec des iframes, par exemple une vidéo YouTube, une publication, une carte. Une page dans une page, tout le monde s'en sert.
- **Pistes à évaluer** : une liste de sites permis, déclarée dans le fichier ; un iframe enfermé par défaut (`sandbox`) ; une vidéo qui ne se charge qu'au toucher du visiteur (une « façade », pas de pistage avant le clic) ; le titre obligatoire pour le lecteur d'écran.

### 2. Le champ mot de passe

- **Aujourd'hui** : seules les pages de compte que fabrique le moteur (`/account/…`) ont un champ mot de passe. Une page d'auteur ne peut pas en demander un.
- **Pourquoi Claude l'a limité** : une page ne doit pas pouvoir récolter des mots de passe (l'hameçonnage). Le moteur les garde lui-même, hachés.
- **Mon objection** : depuis que je connais l'informatique, il y a un champ mot de passe dans tous les formulaires, dans les réseaux sociaux comme dans n'importe quelle application. C'est un des champs les plus importants.
- **Pistes à évaluer** : les cas où un auteur en a vraiment besoin hors des pages de compte (changer son mot de passe, protéger un document, confirmer une action) ; un `Input(type: password)` que le moteur encadre :
  - jamais gardé en clair dans les valeurs de la page ;
  - envoyé seulement au serveur de l'auteur, en HTTPS ;
  - jamais journalisé ;
  - le collage permis et un bouton « montrer » ;
  - les règles de longueur des recommandations publiques (NIST SP 800-63B, ANSSI ou NCSC) ;
  - compatible avec les gestionnaires de mots de passe.

### 3. Lire les données d'un autre serveur (`fetch`)

- **Aujourd'hui** : `Data(from:)` ne lit que le serveur d'où vient la page.
- **Pourquoi Claude l'a limité** : « chez soi d'abord », et la vie privée (le navigateur du visiteur qui contacte un autre site lui donne son adresse IP).
- **Mon objection** : `fetch` est une des choses les plus importantes de JavaScript : la météo, une carte, le cours d'une monnaie, toutes les API.
- **Pistes à évaluer** :
  - (a) le serveur de l'auteur va chercher les données et les garde un moment, avec une liste de sites permis et les clés d'API rangées sur le serveur, jamais dans la page ;
  - (b) le navigateur lit directement des sites déclarés dans le fichier ;
  - (c) les deux.

  Contraintes à vérifier : CORS, `Content-Security-Policy`, le RGPD pour l'adresse IP, les clés d'API exposées, les quotas, les pannes, la page sans JavaScript.

### 4. Les transformations 3D libres (`transform 3D`, `perspective`)

- **Aujourd'hui** : la page entière peut tourner (`Relief(tilt:)`, `flip`), mais pas un élément seul avec un effet 3D libre.
- **Pourquoi Claude l'a limité** : la 3D s'active exprès dans le fichier, une page reste lisible, et le mouvement peut rendre malade.
- **Mon avis** : je n'en ai pas, on le reverra avec la 3D.
- **À évaluer** : les effets utiles (une carte qui se retourne, une inclinaison au survol), les contraintes (troubles vestibulaires, `prefers-reduced-motion`, le texte tourné et le lecteur d'écran, les performances sur un téléphone), et une forme sûre.

### 5. Importer des modules et des paquets venus d'ailleurs

- **Aujourd'hui** :
  - `import "commun.holo"` importe un autre fichier HoloCode ;
  - `module "x.wasm"` charge un module WebAssembly enfermé ;
  - il n'existe pas de catalogue de paquets, ni de JavaScript extérieur.
- **Pourquoi Claude l'a limité** : il avait écrit « pas de code extérieur ». Précision de Claude : un module extérieur *enfermé* est déjà permis ; ce qui est refusé, c'est du code non enfermé.
- **Mon objection** : tôt ou tard, il faudra importer des modules, et je ne comprends pas cette limite.
- **À évaluer** :
  - les contraintes d'un système de paquets : les attaques par la chaîne d'approvisionnement (les incidents de npm, `event-stream`, `ua-parser-js`, et la porte dérobée de `xz`), les empreintes d'intégrité, les versions figées, les licences, le poids, les droits d'un module ;
  - le modèle le plus sûr et le plus simple : un catalogue, une adresse avec son empreinte, ou une copie rangée dans le projet.

### 6. Dessiner librement (Canvas 2D)

- **Aujourd'hui** :
  - `Drawing` fait un dessin vectoriel déclaré (rectangles, ronds, traits, tracés) ;
  - un module peut rendre une liste de formes vérifiées ;
  - un auteur ne peut pas écrire de code de dessin trait par trait.
- **Pourquoi Claude l'a limité** : aucun code libre dans une page.
- **Mon objection** : imagine que je crée un site de dessin où les gens ne peuvent pas dessiner. Ça n'aurait aucun sens.
- **Précision de Claude** : il confondait deux choses. Ce qui est refusé, c'est le code de dessin écrit par l'*auteur*. Un *visiteur* qui dessine au doigt ou à la souris, c'est autre chose : un bloc « zone de dessin » qui produit des traits (des données) que la page garde, envoie ou partage.
- **À évaluer** :
  - les contraintes d'une zone de dessin : l'accessibilité (une alternative au dessin pour qui ne voit pas ou n'utilise que le clavier), le conflit entre le doigt et le défilement sur un téléphone, la pression du stylet, l'annulation, le poids des traits gardés, l'export en PNG ou en SVG, la modération si les dessins sont partagés ;
  - une forme HoloCode.

### 7. Les notifications « push », envoyées par le serveur quand la page est fermée

- **Aujourd'hui** : une notification locale, sur permission, tant que la page est ouverte. Rien quand la page est fermée.
- **Pourquoi Claude l'a limité** : le « Web Push » passe par le service de notifications du navigateur (celui de Google pour Chrome, de Mozilla pour Firefox, d'Apple pour Safari). Ce sont des services extérieurs.
- **Mon objection** : je n'aime pas passer par des services extérieurs.
- **À vérifier précisément** :
  - le Web Push avec des clés VAPID (RFC 8292) demande-t-il un compte ou un contrat chez Google, Apple ou Mozilla ? Ou le serveur de l'auteur fabrique-t-il ses propres clés, le message étant chiffré de bout en bout (RFC 8291), et le service du navigateur ne faisant que le transporter ?
  - est-ce compatible avec « aucun prestataire obligatoire » ?
  - les limites sur iPhone : seulement pour une application web installée, depuis iOS 16.4 ;
  - le bruit des demandes de permission, et le RGPD ;
  - les alternatives sans service extérieur : le direct tant que la page est ouverte (déjà construit), un e-mail envoyé par le serveur de l'auteur.

### 8. Payer sur la page

- **Aujourd'hui** : aucun paiement.
- **Pourquoi Claude l'a limité** : il faudrait un prestataire de paiement.
- **Mon objection** : techniquement, payer sur la page est possible si l'auteur a bien fait son serveur.
- **À vérifier** :
  - la norme PCI DSS : peut-on recevoir des numéros de carte sur son propre serveur, et à quel prix ?
  - l'authentification forte (DSP2, 3-D Secure) ;
  - l'API Payment Request du W3C : que fait-elle, et que ne fait-elle pas ?
  - les moyens sans prestataire de cartes : un virement, le mobile money (M-Pesa, Orange Money, Airtel Money, très utilisés en Afrique), les cryptomonnaies ;
  - une forme qui garde « chez soi d'abord » : le prestataire choisi par l'auteur, réglé sur son serveur, facultatif, jamais obligatoire.

## Le format de ta réponse, pour chaque fonction

```
### N. <la fonction>
- Contraintes trouvées (avec leurs sources) :
- Contraintes imaginées (sans source) :
- Verdict : garder limité / ouvrir entièrement / ouvrir sous conditions
- Forme proposée en HoloCode (si tu ouvres) :
- Risques de ta proposition :
- Ce que je n'ai pas pu vérifier :
```

Merci.
