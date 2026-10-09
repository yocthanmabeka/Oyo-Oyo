# Réponse de Gemini : les huit fonctions que HoloCode limite exprès

- Réponse au prompt `docs/05-discussions/prompts/2026-10-09-contraintes-des-huit-limites.md`.
- Transmise par Yocthan le 2026-10-09. Rangée telle quelle par la session Claude du PC : Gemini n'a pas envoyé lui-même.
- Le texte de Gemini est un avis, pas une décision. Les sources citées n'ont pas été vérifiées une à une au moment du rangement.

---

Bonjour Yocthan. Voici l'analyse indépendante et détaillée des huit fonctions, fonction par fonction, rédigée en français simple conformément à tes règles.

---

### 1. Une page dans la page (`iframe`, `embed`, `object`)

* **Contraintes trouvées (avec leurs sources) :**
* **Sécurité (Clickjacking / Détournement de clic) :** L'OWASP (*Open Web Application Security Project*, guide *Clickjacking Defense Cheat Sheet*) documente qu'intégrer une page tierce ou se laisser intégrer sans contrôle permet de superposer des boutons transparents pour tromper le visiteur.
* **Isolation et permissions :** La spécification HTML du W3C/WHATWG (*HTML Living Standard*, section `The iframe element`) impose l'attribut `sandbox` pour neutraliser par défaut les scripts, formulaires, popups et accès au stockage local de la page intégrée.
* **Pistage et vie privée :** Le W3C et les régulateurs européens (CNIL, lignes directrices sur les traceurs/cookies) rappellent que le chargement direct d'une ressource tierce transmet immédiatement l'adresse IP et dépose des identifiants de suivi sans consentement préalable.
* **Accessibilité :** La règle WCAG 2.1 (Critère de succès 4.1.2 *Name, Role, Value*) exige qu'une balise d'intégration possède obligatoirement un attribut descriptif accessible (`title`) pour être lue par les lecteurs d'écran.
* **Poids et empreinte mobile :** Le projet Chromium / web.dev (*Lighthouse audit: Third-party facades*, mise à jour continue) documente qu'un lecteur YouTube ou une carte interactive complets téléchargent entre 500 Ko et 3 Mo de scripts dès l'ouverture. La recommandation industrielle est l'usage d'une « façade » (afficher une image fixe locale, et ne charger le composant externe qu'au toucher).

* **Contraintes imaginées (sans source) :**
* *Contrainte imaginée, sans source :* Dans la vue 3D en points d'HoloCode, projeter une page web externe qui bouge et s'exécute librement peut faire chuter le taux de rafraîchissement d'un smartphone d'entrée de gamme, sans moyen pour le moteur Rust de limiter le nombre de pixels redessinés par la page intégrée.

* **Verdict :** **Ouvrir sous conditions**.
Ouvrir les intégrations externes, mais jamais sous forme de code libre ou de fenêtre non déclarée. Imposer la façade par défaut, le bac à sable technique (`sandbox`), le titre accessible obligatoire et une liste de domaines autorisés.
* **Forme proposée en HoloCode :**

```holo
Embed(
  title: "Vidéo du projet",
  source: "https://video.exemple.org/embed/123",
  allow: [fullscreen],
  facade: "apercu.svg",
  weight: 20KB,
)
```

* **Risques de ta proposition :**
* Si le site distant change sa politique ou tombe en panne, un cadre blanc disgracieux apparaît sur la page.
* L'intégration d'un site non adapté aux écrans étroits peut forcer un défilement horizontal désagréable sur smartphone.

* **Ce que je n'ai pas pu vérifier :**
* Le comportement précis du moteur WebGPU sur smartphone économique lorsqu'une surface externe (`iframe` HTML) est déformée ou mise en relief par `Relief(tilt:)`.

---

### 2. Le champ mot de passe

* **Contraintes trouvées (avec leurs sources) :**
* **Règles de composition et collage :** Le NIST (*National Institute of Standards and Technology*, publication spéciale *SP 800-63B - Digital Identity Guidelines*, section 5.1.1.2) ainsi que l'ANSSI (guide des recommandations sur les mots de passe) recommandent d'autoriser impérativement le collage (pour gestionnaires de mots de passe), d'offrir une option d'affichage en clair temporaire (« révéler ») et d'interdire les troncatures ou règles arbitraires empêchant les phrases de passe longues (autoriser au moins 64 caractères).
* **Hameçonnage (Phishing) :** Les filtres de navigation sécurisée (comme *Google Safe Browsing* ou *Microsoft SmartScreen*) signalent et bloquent automatiquement les sites dont les pages non certifiées ou non identifiées collectent des identifiants dans des formulaires génériques non sécurisés.

* **Contraintes imaginées (sans source) :**
* *Contrainte imaginée, sans source :* Si un auteur malveillant conçoit une fausse page de banque ou de réseau social avec HoloCode, il pourrait tenter de capturer des mots de passe en clair par les signaux de règles si le moteur autorisait la lecture de la chaîne de texte du champ dans l'arbitre.

* **Verdict :** **Ouvrir sous conditions**.
Permettre `Input(type: password)`, mais le champ doit être une boîte noire pour la page : la valeur tapée n'est accessible à aucun bloc d'affichage ni règle d'évaluation locale, elle ne peut être qu'empaquetée de manière chiffrée lors de la soumission d'un formulaire `Form` vers le serveur de l'auteur en HTTPS.
* **Forme proposée en HoloCode :**

```holo
Input(
  name: MotDePasse,
  type: password,
  label: "Mot de passe",
  allowPaste: true,
  revealable: true,
)
```

* **Risques de ta proposition :**
* Risque que des auteurs conçoivent des formulaires trompeurs incitant des visiteurs à entrer des codes confidentiels, exposant le domaine de l'auteur aux listes de blocage des navigateurs.

* **Ce que je n'ai pas pu vérifier :**
* Le support standardisé des attributs d'auto-remplissage (`autocomplete="current-password"` ou `"new-password"`) par l'ensemble des gestionnaires de mots de passe tiers sur les navigateurs mobiles sans balisage HTML complexe.

---

### 3. Lire les données d'un autre serveur (`fetch`)

* **Contraintes trouvées (avec leurs sources) :**
* **CORS (Cross-Origin Resource Sharing) :** La spécification Fetch du W3C/WHATWG documente que le navigateur d'un visiteur bloque par défaut la lecture d'une réponse issue d'un autre serveur, à moins que le serveur distant n'envoie explicitement l'en-tête HTTP `Access-Control-Allow-Origin`.
* **Fuite de clés secrètes :** Les bonnes pratiques de l'OWASP rappellent que stocker une clé d'API privée dans un fichier distribué au client (navigateur) équivaut à la rendre publique.
* **Vie privée et RGPD :** La Cour de justice de l'Union européenne (arrêt *Fashion ID*, C-40/17, 2019) a établi qu'intégrer une requête directe vers un tiers transmettant l'adresse IP du visiteur sans base légale engage la responsabilité conjointe de l'éditeur du site.

* **Contraintes imaginées (sans source) :**
* *Contrainte imaginée, sans source :* Si le navigateur interrogeait directement une API météo ou financière qui tombe en panne, le déterminisme de la page HoloCode serait brisé (la page afficherait un état d'erreur imprévu, cassant la parité entre visiteurs).

* **Verdict :** **Ouvrir sous conditions (via le serveur de l'auteur en priorité)**.
Suivre la piste (a) : c'est le serveur de l'auteur (`holo serve`) qui va chercher la donnée tierce, utilise ses clés secrètes stockées localement sur le PC, met la réponse en cache (par exemple pour 10 minutes), et la sert à la page. La lecture directe par le navigateur (piste b) ne doit être tolérée que pour des données publiques ouvertes avec CORS validé.
* **Forme proposée en HoloCode :**

```holo
Data(
  name: Meteo,
  source: "https://api.meteo-publique.org/paris",
  cached: 15min,
  fallback: "Météo indisponible",
)
```

* **Risques de ta proposition :**
* Si le serveur de l'auteur est hébergé chez lui sur son PC et que sa connexion Internet coupe, le cache local continuera de servir la dernière valeur connue au lieu de bloquer l'affichage.

* **Ce que je n'ai pas pu vérifier :**
* Le comportement précis des quotas d'appels des API gratuites lorsque plusieurs visiteurs interrogent le serveur local d'un auteur simultanément.

---

### 4. Les transformations 3D libres (`transform 3D`, perspective)

* **Contraintes trouvées (avec leurs sources) :**
* **Santé et accessibilité (Troubles vestibulaires) :** Le W3C (lignes directrices WCAG 2.1, Critère 2.3.3 *Animation from Interactions*) et la spécification CSS Media Queries Level 5 (`prefers-reduced-motion`) stipulent que les effets visuels de bascule spatiale ou de parallaxe provoquent nausées et désorientation chez les personnes souffrant de troubles vestibulaires si aucune option d'inhibition n'est fournie.
* **Lisibilité et rendu typographique :** La documentation de rendu de Mozilla (MDN Web Docs, section sur `transform-style: preserve-3d`) rappelle qu'une surface textuelle inclinée subit une rastérisation avec anticrénelage sous-pixel dégradé, nuisant à la lisibilité des petites polices sur écrans basse résolution.
* **Lecteurs d'écran :** Une transformation CSS 3D géométrique n'altère pas l'arbre d'accessibilité (l'ordre de lecture reste linéaire), mais retourner une carte à 180° peut afficher visuellement un texte tout en masquant le recto sans prévenir l'utilisateur malvoyant.

* **Contraintes imaginées (sans source) :**
* *Contrainte imaginée, sans source :* Autoriser des coordonnées de rotation 3D arbitraires sur chaque bloc individuel dans HoloCode risquerait de créer des intersections de surfaces qui clignotent (*Z-fighting*) dans le moteur WebGPU lors du zoom en profondeur.

* **Verdict :** **Ouvrir sous conditions**.
Interdire les transformations matricielles libres continues sur n'importe quel bloc, mais introduire des blocs d'effets finis et déclaratifs (par exemple un bloc de carte à deux faces réversible, ou une bascule d'angle déclarée). Désactiver automatiquement le mouvement si `prefers-reduced-motion` est actif.
* **Forme proposée en HoloCode :**

```holo
CardFlip(
  front: [ Text("Recto du tableau") ],
  back: [ Text("Détails et prix") ],
  trigger: tap,
)
```

* **Risques de ta proposition :**
* L'effet visuel peut être moins spectaculaire que du CSS3 expérimental libre, mais il garantit la stabilité typographique et ne rend pas malade.

* **Ce que je n'ai pas pu vérifier :**
* L'impact exact sur la consommation mémoire GPU du maintien simultané en mémoire des textures du recto et du verso sur un smartphone disposant de moins de 3 Go de RAM globale.

---

### 5. Importer des modules et des paquets venus d'ailleurs

* **Contraintes trouvées (avec leurs sources) :**
* **Attaques par la chaîne d'approvisionnement (Supply Chain Attacks) :** Les incidents documentés de l'écosystème npm (*event-stream* en 2018, injection malveillante dans *ua-parser-js* en 2021) et la porte dérobée découverte dans les outils de compression *xz/liblzma* (CVE-2024-3094 en 2024) démontrent que déléguer aveuglément les dépendances à un gestionnaire de paquets centralisé sans vérification cryptographique expose l'ensemble des utilisateurs à du code hostile.
* **Intégrité des sous-ressources (SRI) :** La recommandation du W3C (*Subresource Integrity - W3C Recommendation*) impose l'utilisation d'empreintes cryptographiques (SHA-256 ou SHA-384) pour garantir qu'un fichier téléchargé n'a pas été altéré en transit ou sur un serveur miroir.
* **Sandboxing WebAssembly :** La spécification W3C WebAssembly Core documente que le bac à sable Wasm n'accorde par défaut aucun appel système, mémoire externe ou communication réseau, sauf importation explicite fournie par l'hôte.

* **Contraintes imaginées (sans source) :**
* *Contrainte imaginée, sans source :* Si un auteur HoloCode téléchargeait un module `.wasm` compilé par un tiers qui exécute une boucle sans fin, le fil d'exécution de l'arbitre pourrait se figer si le compteur d'instructions du moteur Rust n'est pas borné.

* **Verdict :** **Ouvrir sous conditions (copie locale ou URL avec empreinte obligatoire)**.
Rejeter le modèle d'un gestionnaire centralisé automatique de type npm avec dépendances transitives infinies. Autoriser l'importation de modules Wasm extérieurs uniquement si le module est soit stocké localement dans le dossier du projet, soit référencé par une adresse HTTPS assortie d'une empreinte cryptographique stricte (`hash: sha256-...`).
* **Forme proposée en HoloCode :**

```holo
module "calcul-physique.wasm" (
  hash: "sha256-a1b2c3d4...",
  maxMemory: 16MB,
  timeout: 50ms,
)
```

* **Risques de ta proposition :**
* Si le développeur du module met à jour son fichier distant sans changer l'URL, l'empreinte ne correspondra plus et le moteur refusera de démarrer la page. C'est contraignant, mais c'est le prix de la sécurité.

* **Ce que je n'ai pas pu vérifier :**
* Le temps exact d'analyse et de validation d'un binaire Wasm de plusieurs mégaoctets sur le moteur Rust embarqué d'un smartphone d'entrée de gamme.

---

### 6. Dessiner librement (`Canvas 2D`)

* **Contraintes trouvées (avec leurs sources) :**
* **Accessibilité des graphismes (WCAG 2.1) :** La règle WCAG (Critère 1.1.1 *Non-text Content*) impose que toute surface graphique non textuelle interactive propose une alternative textuelle équivalente ou une description textuelle des actions effectuées.
* **Conflit tactile et gestuel :** La spécification W3C *Pointer Events Level 3* (propriété CSS `touch-action`) documente que capturer les tracés continus d'un doigt sur écran mobile entre en conflit direct avec le défilement natif de la page (*pan-y*), nécessitant une déclaration explicite pour désactiver le défilement sur la zone active.
* **Poids des données vectorielles :** La sérialisation brute de milliers de coordonnées de points de contact peut rapidement saturer la mémoire et la bande passante si les tracés ne subissent pas de simplification polygonale (comme l'algorithme de Ramer-Douglas-Peucker).

* **Contraintes imaginées (sans source) :**
* *Contrainte imaginée, sans source :* Dans un espace partagé où les visiteurs dessinent ensemble, l'absence de modération locale ou de limitation du débit de tracés pourrait être exploitée pour saturer l'affichage des autres participants en superposant des millions de traits noirs.

* **Verdict :** **Ouvrir sous conditions (bloc de surface de dessin déclaratif)**.
Ne jamais autoriser l'écriture de code de tracé impératif JavaScript (`ctx.beginPath()`). Fournir un bloc de dessin interactif (`DrawingCanvas`) géré par le moteur Rust : l'utilisateur dessine au doigt, à la souris ou au stylet, le moteur lisse les traits, les enregistre sous forme de tableau de données géométriques, et permet leur export en SVG ou PNG.
* **Forme proposée en HoloCode :**

```holo
CanvasInput(
  name: ZoneDessin,
  width: 100%,
  height: 300px,
  tool: pen,
  color: "#E4572E",
  strokeWidth: 4px,
  alt: "Zone de dessin interactive",
)
```

* **Risques de ta proposition :**
* Les personnes ne pouvant utiliser ni souris ni écran tactile (navigation exclusivement au clavier) ne pourront pas dessiner directement, à moins de prévoir un mode de saisie de coordonnées assisté.

* **Ce que je n'ai pas pu vérifier :**
* La précision de restitution de la pression des stylets numériques (Apple Pencil, Samsung S-Pen) lors du passage à travers l'interface standardisée de WebAssembly.

---

### 7. Les notifications « push » (quand la page est fermée)

* **Contraintes trouvées (avec leurs sources) :**
* **Architecture et protocoles standardisés :** Les normes RFC 8291 (*Message Encryption for Web Push*) et RFC 8292 (*VAPID - Voluntary Application Server Identification for Web Push*) définissent le fonctionnement du Web Push.
  * *Réponse formelle sur les intermédiaires :* **Il n'y a pas besoin de contrat ni de compte payant chez Google, Apple ou Mozilla.** Le serveur de l'auteur génère gratuitement sa propre paire de clés cryptographiques VAPID.
  * *Rôle des relais :* Cependant, le message chiffré transite **obligatoirement** par le service de notification du système d'exploitation du client (FCM pour Google/Chrome, APNs pour Apple/Safari, Mozilla Push Service pour Firefox). Le message est chiffré de bout en bout : le relais transporte des octets illisibles sans pouvoir en lire le contenu.
* **Restrictions strictes sur iPhone/iOS :** La documentation officielle d'Apple (*WebKit Features in Safari 16.4*, mars 2023) spécifie que sur iOS/iPadOS, les notifications Web Push ne fonctionnent **que si le site a été installé sur l'écran d'accueil** par l'utilisateur comme une application web (PWA), et exige un geste tactile explicite pour accorder la permission.
* **Législation et protection des données (RGPD) :** L'identifiant unique de notification (*PushSubscription endpoint*) constitue une donnée à caractère personnel selon le Comité européen de la protection des données (EDPB), nécessitant un consentement clair et révocable.

* **Contraintes imaginées (sans source) :**
* *Contrainte imaginée, sans source :* Si un serveur personnel hébergé chez un particulier sur une connexion résidentielle change d'adresse IP publique, le maintien des abonnements push actifs peut devenir complexe sans nom de domaine stable.

* **Verdict :** **Ouvrir sous conditions**.
Le Web Push respecte la règle « chez soi d'abord » : l'auteur génère ses clés sur son propre PC avec `holo serve`, sans payer aucun prestataire. L'intermédiaire technique est imposé par les fabricants de téléphones pour préserver la batterie de l'appareil. En alternative sans service tiers, conserver les notifications locales quand l'onglet est ouvert, ou l'envoi d'e-mails directement par le serveur.
* **Forme proposée en HoloCode :**

```holo
PushService(
  name: Alertes,
  topic: "nouvelles_commandes",
  fallback: local,
)
```

* **Risques de ta proposition :**
* Les fenêtres de demande d'autorisation de notifications sont perçues comme très intrusives si elles apparaissent dès l'ouverture de la page.
* Les notifications ne sonneront pas sur les téléphones Apple si l'utilisateur n'a pas épinglé la page à son écran d'accueil.

* **Ce que je n'ai pas pu vérifier :**
* La politique de conservation des messages en file d'attente par les relais d'Apple (APNs) lorsque le téléphone d'un destinataire reste éteint pendant plusieurs semaines consécutives.

---

### 8. Payer sur la page

* **Contraintes trouvées (avec leurs sources) :**
* **Réglementation bancaire et sécurité (PCI DSS) :** Le standard international *PCI DSS v4.0* (*Payment Card Industry Data Security Standard*, Conseil des normes de sécurité PCI) interdit formellement de stocker, traiter ou transmettre des numéros de carte bancaire bruts sur son propre serveur sans obtenir une certification de sécurité d'infrastructure de niveau bancaire (dont les audits coûtent plusieurs dizaines de milliers d'euros par an). Recevoir directement un numéro de carte sur un PC personnel chez soi est hors d'atteinte légale et financière.
* **Authentification forte du client (DSP2 / 3-D Secure) :** La Directive européenne sur les services de paiement (DSP2 / RTS sur l'authentification forte) impose un défi d'authentification biométrique ou bancaire à deux facteurs pour toute transaction en ligne par carte.
* **API W3C Payment Request :** La spécification *Payment Request API* du W3C n'est pas un système de traitement financier : c'est uniquement une interface graphique du navigateur qui collecte les adresses et transmet de façon sécurisée un moyen de paiement vers un processeur certifié.
* **Solutions souveraines sans carte :** Les flux de Mobile Money (M-Pesa, Orange Money, Airtel Money) reposent sur des protocoles d'initiation de paiement par serveur (USSD push ou redirection d'API) où le numéro de téléphone valide la transaction directement sur la carte SIM de l'acheteur, sans toucher à la norme PCI DSS des cartes bancaires. Le virement bancaire SEPA ou le règlement à la livraison n'exigent aucun tiers.

* **Contraintes imaginées (sans source) :**
* *Contrainte imaginée, sans source :* Permettre à un auteur néophyte d'écrire son propre formulaire de carte bancaire sans tiers conduirait inévitablement à des fuites de données massives en cas de piratage du PC de l'auteur.

* **Verdict :** **Ouvrir sous conditions (délégation bancaire déclarative)**.
Interdire formellement la saisie de numéros de carte bancaire en clair dans HoloCode. Ouvrir le paiement sous forme d'une commande déclarative : l'auteur configure sur son serveur `holo serve` le mode de son choix (un prestataire comme Stripe s'il le souhaite, son compte Mobile Money M-Pesa/Orange Money, ou un simple bon de virement/livraison). Le bouton dans la page ne fait qu'initier la commande sans jamais toucher aux données bancaires brutes.
* **Forme proposée en HoloCode :**

```holo
PaymentButton(
  name: PayerCommande,
  amount: panier.total,
  currency: "EUR",
  methods: [mobile_money, transfer, provider],
  onSuccess: panier.clear(),
)
```

* **Risques de ta proposition :**
* Les prestataires de cartes exigent des vérifications d'identité (KYC) pour virer les fonds vers un compte bancaire professionnel.
* Les interfaces Mobile Money varient d'un pays à l'autre et imposent des adaptateurs réseau distincts sur le serveur Rust.

* **Ce que je n'ai pas pu vérifier :**
* Les barèmes exacts de commissions appliqués par les opérateurs de Mobile Money aux particuliers hébergeant leur propre boutique en ligne sans statut d'entreprise immatriculée.

---

### Synthèse des verdicts

| Fonction | Verdict | Justification en une phrase |
| --- | --- | --- |
| **1. Page dans la page (`iframe`)** | **Ouvrir sous conditions** | Indispensable pour intégrer des vidéos et cartes, à condition d'imposer une façade légère, un bac à sable strict et un titre accessible. |
| **2. Champ mot de passe** | **Ouvrir sous conditions** | Nécessaire pour les confirmations et protections, mais le texte doit rester une boîte noire opaque transmise uniquement en HTTPS au serveur. |
| **3. Données d'un autre serveur (`fetch`)** | **Ouvrir sous conditions** | Le serveur de l'auteur doit agir comme relais avec cache local pour protéger les clés secrètes, la vie privée et contourner les blocages CORS. |
| **4. Transformations 3D libres** | **Ouvrir sous conditions** | Proposer des composants à bascule finis (comme une carte réversible) plutôt que des angles libres, et désactiver le mouvement sur consigne d'accessibilité. |
| **5. Modules extérieurs** | **Ouvrir sous conditions** | Conserver le bac à sable WebAssembly hermétique, et imposer une copie locale du binaire ou une empreinte cryptographique SHA-256 obligatoire. |
| **6. Dessiner librement** | **Ouvrir sous conditions** | Fournir un bloc de capture vectorielle interactif où le visiteur dessine au doigt, sans jamais autoriser de scripts impératifs libres. |
| **7. Notifications push** | **Ouvrir sous conditions** | Compatible avec l'hébergement personnel grâce aux clés VAPID gratuites créées par l'auteur, malgré l'obligation d'un relais système tiers. |
| **8. Payer sur la page** | **Ouvrir sous conditions** | Banaliser le déclenchement de paiement (Mobile Money, virement, prestataire optionnel) en interdisant formellement la manipulation de numéros de carte bruts. |
