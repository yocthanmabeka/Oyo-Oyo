# Les huit fonctions « limitées exprès » — avis et contraintes

**Pour :** Yocthan Mabeka, projet Holoverse / HoloCode (`yocthanmabeka/Oyo-Oyo`)
**Date :** 9 octobre 2026
**Rédigé par :** Claude, dans une conversation séparée, à partir de la description du projet (sans lire le code)

> **À savoir avant de lire.** Je suis moi aussi Claude, le même type de modèle que la session qui programme avec toi. Mon avis n'est donc pas totalement « indépendant » : j'ai cherché les contraintes sur Internet plutôt que de me fier à ce qu'on m'avait dit, mais pour un vrai regard extérieur, un humain (un développeur web ou un spécialiste sécurité) reste le meilleur second avis.
>
> Les sources sont listées à la fin, avec leur lien et leur date quand la page en donne une. Quand une contrainte n'a pas de source, c'est écrit : « contrainte imaginée, sans source ».

---

## En résumé

| Fonction | Verdict |
|---|---|
| 1. Une page dans la page | **Ouvrir sous conditions** |
| 2. Le champ mot de passe | **Ouvrir sous conditions** |
| 3. Lire un autre serveur | **Ouvrir sous conditions** (par le serveur de l'auteur) |
| 4. Les effets 3D sur un élément | **Ouvrir sous conditions** (un petit ensemble) |
| 5. Les paquets venus d'ailleurs | **Ouvrir sous conditions** (copie rangée + empreinte) |
| 6. Dessiner | **Ouvrir** la zone de dessin du visiteur ; garder limité le code de dessin |
| 7. Les notifications push | **Ouvrir sous conditions** (facultatif) |
| 8. Payer | **Ouvrir sous conditions**, sans jamais recevoir de numéro de carte |

Sur le fond, **tu as raison sur les huit** : aucune ne doit rester fermée. Mais la session Claude Code a aussi raison sur les risques. La bonne réponse est presque toujours la même : **ouvrir, mais c'est le moteur qui tient les règles de sécurité, pas l'auteur.** C'est exactement l'esprit de HoloCode.

---

### 1. Une page dans la page (iframe, embed, object)

- **Contraintes trouvées (avec leurs sources) :**
  - **Le clickjacking** : une page d'attaquant charge un site dans un cadre invisible et fait cliquer l'utilisateur sur des boutons qu'il ne voit pas [S1]. La protection moderne est l'en-tête `frame-ancestors`, qui dit au navigateur qui a le droit de mettre la page dans un cadre [S1]. Note : ce risque concerne surtout **les pages HoloCode mises dans le cadre d'un autre site**, pas l'inverse. Le moteur devrait donc protéger les pages HoloCode par défaut.
  - **L'attribut `sandbox`** (cadre enfermé) bloque par défaut les scripts et les fenêtres [S2]. Mais il n'empêche pas tout : des attaquants l'utilisent même pour contourner certaines protections [S3].
  - **Le pistage** : une vidéo YouTube intégrée normalement pose des cookies tiers, même si le visiteur ne la regarde pas [S4]. Le « mode confidentialité renforcée » (`youtube-nocookie.com`) n'est **pas** une vraie solution : il ne pose pas de cookie au chargement, mais garde un identifiant dans le stockage du navigateur [S5].
  - **La façade** (une image + un bouton lecture, la vraie vidéo ne se charge qu'au clic) : rien ne part chez YouTube avant le choix du visiteur, et la page est plus rapide [S6].
  - **Le titre obligatoire** : les lecteurs d'écran annoncent le titre d'un cadre ; un cadre sans titre est un échec du critère WCAG 4.1.2 (niveau A) [S7].
- **Contraintes imaginées (sans source) :**
  - L'image de la façade elle-même vient souvent du site de la vidéo (la miniature YouTube). La charger directement donnerait déjà l'adresse IP du visiteur. → Le serveur de l'auteur doit aller chercher la miniature et la servir lui-même.
  - Un site permis aujourd'hui peut changer de propriétaire demain (nom de domaine racheté).
  - Un cadre peut demander la caméra, le micro ou la position : à interdire par défaut.
- **Verdict : ouvrir sous conditions.**
- **Forme proposée en HoloCode :**
  ```
  Allowed(embeds: ["youtube-nocookie.com", "openstreetmap.org"])

  Embed(from: "https://www.youtube-nocookie.com/embed/ABC123",
        title: "Vidéo : présentation du projet",
        load: on-tap)
  ```
  Règles tenues par le moteur :
  - `title` obligatoire, sinon le moteur refuse ;
  - seulement des sites listés dans `Allowed(embeds:)` ;
  - `load: on-tap` (la façade) **par défaut** pour tout site extérieur ; la miniature passe par le serveur de l'auteur ;
  - cadre enfermé (`sandbox`) par défaut, sans caméra, micro, position, ni plein écran sauf permission écrite ;
  - sans JavaScript : la façade devient un simple lien vers la vidéo ;
  - les pages HoloCode elles-mêmes sont servies avec `frame-ancestors 'self'` (personne ne peut les mettre dans un cadre sans permission).
- **Risques de ma proposition :** une fois que le visiteur touche « lecture », il est bien chez YouTube et pisté. Certains lecteurs ne marchent pas dans un cadre trop enfermé : il faudra peut-être des réglages par site.
- **Ce que je n'ai pas pu vérifier :** quels réglages `sandbox` précis acceptent YouTube, Vimeo et OpenStreetMap sans casser ; si le moteur peut garder la parité sur un téléphone ancien avec un lecteur vidéo extérieur.

---

### 2. Le champ mot de passe

- **Contraintes trouvées (avec leurs sources) :**
  - **NIST SP 800-63B-4**, finalisé en août 2025 : au moins 15 caractères si le mot de passe est le seul facteur, au moins 8 s'il y a toujours un second facteur ; accepter au moins 64 caractères [S8][S9].
  - Le NIST **exige** d'accepter les gestionnaires de mots de passe et le remplissage automatique ; bloquer le collage est considéré comme une erreur [S9]. Pas de changement forcé périodique [S10].
  - Il faut aussi vérifier chaque nouveau mot de passe contre une liste de mots de passe connus ou volés [S9].
- **Contraintes imaginées (sans source) :**
  - **L'hameçonnage** : un auteur peut fabriquer une page qui ressemble à la connexion d'un autre site. Ce risque existe sur tout le web, HoloCode ne peut pas l'empêcher totalement, mais il peut empêcher le pire : que le mot de passe parte ailleurs que chez l'auteur.
  - Un auteur qui garde les mots de passe en clair dans SQLite par négligence.
  - Un mot de passe qui finit dans les journaux du serveur, dans l'historique de la page ou dans une valeur partagée en direct.
- **Verdict : ouvrir sous conditions.** Les cas réels hors des pages de compte existent : protéger un document, confirmer une action sensible (supprimer son compte), un espace membre avec ses propres règles.
- **Forme proposée en HoloCode :**
  ```
  Form(to: server, name: "protéger") {
    Input(type: password, purpose: new, label: "Mot de passe du document")
    Button("Protéger")
  }
  ```
  Règles tenues par le moteur :
  - `purpose:` obligatoire (`current`, `new`, `confirm`) → le moteur pose les bons repères pour les gestionnaires de mots de passe ;
  - la valeur n'entre **jamais** dans `State`, ni dans une règle, ni dans l'historique, ni dans une valeur partagée ;
  - elle ne part **que** vers le serveur de l'auteur (même adresse), en HTTPS, jamais vers un autre site ;
  - côté serveur, elle arrive comme un type spécial (`Secret`) que l'auteur ne peut que **hacher (Argon2id)** ou **comparer** : impossible de l'écrire en clair ou de la journaliser ;
  - collage permis, bouton « montrer », longueurs du NIST, liste de mots de passe interdits fournie par le moteur ;
  - marche sans JavaScript (simple formulaire envoyé).
- **Risques de ma proposition :** ça ne protège pas d'un faux site complet ; c'est plus de travail pour le moteur (le type `Secret`).
- **Ce que je n'ai pas pu vérifier :** les recommandations actuelles exactes de l'ANSSI et du NCSC (je n'ai lu que des résumés qui les citent) ; je me suis appuyé sur le NIST.

---

### 3. Lire les données d'un autre serveur (fetch)

- **Contraintes trouvées (avec leurs sources) :**
  - **L'adresse IP est une donnée personnelle** (arrêt Breyer de la Cour de justice de l'UE). Le 20 janvier 2022, le tribunal de Munich a condamné un site à 100 € de dommages pour avoir fait envoyer l'adresse IP de ses visiteurs à Google (Google Fonts), sans consentement [S11][S12]. Point clé : le tribunal a jugé que c'était évitable, puisque le site **pouvait servir les polices lui-même** [S13].
  - Conséquence directe : l'option (b), où le navigateur du visiteur contacte un autre site, pose exactement ce problème. L'option (a), où le serveur de l'auteur va chercher les données, l'évite.
- **Contraintes imaginées (sans source) :**
  - **Clés d'API** : placées dans la page, n'importe qui peut les lire et les voler. Elles doivent rester sur le serveur.
  - **Le serveur détourné** : si une page peut demander au serveur de l'auteur « va chercher telle adresse », un attaquant pourrait s'en servir pour atteindre des machines internes (son routeur, son réseau). → liste de sites permis stricte, et refus des adresses privées.
  - **Les pannes et les quotas** : la météo tombe, ou la limite d'appels est atteinte.
  - Le déterminisme : une donnée extérieure change. Mais c'est déjà le cas des données de la base SQLite : la **page** reste déterminée, les **données** non.
- **Verdict : ouvrir sous conditions, par le serveur de l'auteur (option a).** L'option (b) reste fermée par défaut.
- **Forme proposée en HoloCode :**
  ```
  Allowed(sources: ["api.open-meteo.com"])

  Data(name: meteo,
       from: "https://api.open-meteo.com/v1/forecast?latitude=-4.3&longitude=15.3",
       via: server,
       keep: 10min,
       key: secret("METEO_KEY"))
  ```
  Règles tenues par le moteur :
  - seulement des sites listés ;
  - la clé vit dans la configuration du serveur, jamais dans le fichier ni dans la page ;
  - le serveur garde la réponse un moment (`keep:`), ce qui protège les quotas ;
  - en cas de panne, il montre la dernière valeur avec son âge (« météo d'il y a 2 h ») ;
  - marche sans JavaScript, puisque c'est le serveur qui fabrique la page ;
  - « chez soi d'abord » respecté : la page tourne même si le site extérieur est en panne.
- **Risques de ma proposition :** le serveur de l'auteur fait plus de travail et devient le point par où tout passe ; les données « en direct à la seconde » (cours de bourse) sont moins adaptées.
- **Ce que je n'ai pas pu vérifier :** l'état actuel de la jurisprudence européenne au-delà de l'arrêt de Munich de 2022 ; les lois de protection des données de la RDC.

---

### 4. Les transformations 3D libres (un élément qui tourne, la perspective)

- **Contraintes trouvées (avec leurs sources) :**
  - **WCAG 2.3.3** (niveau AAA) : une animation déclenchée par l'utilisateur doit pouvoir être désactivée, sauf si elle est essentielle [S14]. Les troubles vestibulaires (oreille interne) donnent vertiges, nausées et maux de tête [S14].
  - Le réglage « réduire les animations » existe dans macOS, Windows, iOS et Android, et la page peut le lire (`prefers-reduced-motion`) [S15].
- **Contraintes imaginées (sans source) :**
  - Le survol n'existe pas au doigt → une « inclinaison au survol » casserait la parité.
  - Une carte retournée : le lecteur d'écran pourrait lire les deux faces en même temps, ou aucune.
  - Un texte tourné en perspective devient flou ou illisible.
  - Beaucoup d'éléments 3D en même temps peuvent faire chauffer un vieux téléphone.
- **Verdict : ouvrir sous conditions, avec un petit ensemble d'effets nommés**, pas de 3D libre. Je suis d'accord avec toi : la 3D libre attend le plan 3D. Mais deux effets très utilisés peuvent venir avant.
- **Forme proposée en HoloCode :**
  ```
  FlipCard(on: tap) {
    Front { H2("Question") }
    Back  { Text("Réponse") }
  }

  Card(lift: on-hover)
  ```
  Règles tenues par le moteur :
  - `FlipCard` se retourne au toucher, au clic **et** à la touche Entrée (parité) ; le lecteur d'écran ne lit que la face visible ;
  - si « réduire les animations » est activé, le retournement devient un simple fondu ;
  - `lift: on-hover` ne s'applique qu'avec une souris ; au doigt, rien (aucune fonction ne doit en dépendre) ;
  - angles bornés par le moteur, aucun texte ne reste tourné au repos.
- **Risques de ma proposition :** la liste d'effets peut sembler courte aux auteurs ; il faudra résister à l'ajouter effet par effet sans règle.
- **Ce que je n'ai pas pu vérifier :** le coût réel sur le Flip 3 de dix cartes animées à la fois.

---

### 5. Importer des modules et des paquets venus d'ailleurs

- **Contraintes trouvées (avec leurs sources) :**
  - **event-stream (2018)** : l'auteur a passé la main à un inconnu, qui a ajouté une dépendance (`flatmap-stream`) contenant un code chiffré qui volait des bitcoins, visant le portefeuille Copay [S16][S17].
  - **ua-parser-js (octobre 2021)** : le compte de l'auteur a été piraté ; trois versions publiées contenaient des mineurs de cryptomonnaie et des voleurs de mots de passe ; l'attaque a duré environ quatre heures [S18][S19].
  - **xz (CVE-2024-3094, mars 2024)** : un développeur a gagné la confiance du projet pendant plus de deux ans, puis a caché une porte dérobée dans les versions 5.6.0 et 5.6.1 ; elle a été découverte par hasard, sur une lenteur de SSH [S20].
  - Ces trois attaques ont un point commun : **une nouvelle version, installée automatiquement, différente de ce qu'on croyait.**
- **Contraintes imaginées (sans source) :**
  - Un catalogue central demande de la modération, des serveurs, et devient une cible ; il contredit aussi « chez soi d'abord ».
  - Un module devient très lourd et alourdit toutes les pages qui l'utilisent.
  - La licence d'un module interdit l'usage commercial et l'auteur ne le sait pas.
- **Verdict : ouvrir sous conditions — le modèle le plus sûr et le plus simple est « la copie rangée dans le projet, avec son empreinte ».**
- **Forme proposée en HoloCode :**
  ```
  holo add https://exemple.org/graphiques-1.2.0.holo
  ```
  - L'outil télécharge le module, calcule son empreinte (`sha256`), le **copie** dans un dossier du projet (`modules/`) et note l'adresse, la version, l'empreinte, la licence et le poids dans un fichier verrou.
  - Dans la page : `import "modules/graphiques-1.2.0.holo"` — on n'importe **que** ce qui est rangé dans le projet.
  - Aucune mise à jour automatique. `holo update` montre ce qui a changé et demande l'accord.
  - Un module WebAssembly reste enfermé (mémoire, temps, pas de réseau), et ses besoins sont affichés à l'installation.
  - Pas de catalogue pour l'instant. Plus tard, un catalogue pourra n'être qu'une **liste d'adresses avec empreintes**, sans héberger le code.
- **Risques de ma proposition :** l'empreinte protège contre une version changée en silence, pas contre une première version déjà piégée. Les mises à jour de sécurité ne viennent pas toutes seules : il faudra prévenir l'auteur.
- **Ce que je n'ai pas pu vérifier :** comment Deno, Go ou Cargo (des systèmes proches) gèrent ce compromis en détail aujourd'hui.

---

### 6. Dessiner librement (Canvas 2D)

- **Contraintes trouvées (avec leurs sources) :**
  - Les **événements de pointeur** traitent la souris, le stylet et le doigt avec le même code, et donnent la **pression** et l'inclinaison du stylet [S21][S22].
  - Pour qu'on puisse dessiner au doigt, la zone doit dire au navigateur de ne pas faire défiler la page (`touch-action: none`) [S21].
- **Contraintes imaginées (sans source) :**
  - **Le défilement** : si la zone de dessin occupe tout l'écran d'un téléphone, on ne peut plus faire défiler la page. → défilement à deux doigts, ou zone qui ne prend jamais tout l'écran.
  - **L'accessibilité** : une personne aveugle ou au clavier ne peut pas dessiner à main levée. → une alternative : poser des formes au clavier, et un champ « décris ton dessin » quand il est partagé.
  - **Le poids** : des milliers de points par trait ; un dessin peut peser lourd. → lisser et simplifier les traits, plafond de taille.
  - **La modération** : si les dessins sont partagés publiquement, des contenus choquants apparaîtront.
- **Verdict : ouvrir la zone de dessin du visiteur. Garder limité le code de dessin écrit par l'auteur** (ce que le module enfermé couvre déjà).
- **Forme proposée en HoloCode :**
  ```
  Sketch(name: dessin,
         label: "Zone de dessin",
         tools: [pen, eraser, color],
         pressure: on,
         undo: 50,
         max: 2MB,
         export: [png, svg])
  ```
  - Le résultat est une **donnée** (une liste de traits), gardée dans `State`, envoyée au serveur ou partagée en direct.
  - Les mêmes traits redonnent toujours le même dessin (déterminisme).
  - Annuler et refaire au clavier (Ctrl+Z), outils atteignables au clavier, `label` obligatoire.
- **Risques de ma proposition :** une vraie application de dessin (calques, pinceaux) demandera beaucoup plus d'outils ; il faudra décider jusqu'où va le bloc.
- **Ce que je n'ai pas pu vérifier :** la qualité de la pression du stylet sur les Galaxy Z Flip (sans stylet) et sur les navigateurs d'iPhone.

---

### 7. Les notifications « push », quand la page est fermée

- **Contraintes trouvées (avec leurs sources) :**
  - **Pas de compte, pas de contrat.** Avec VAPID (RFC 8292), c'est **le serveur de l'auteur qui fabrique et garde sa propre paire de clés** [S23]. Les services de push ne demandent ni clé d'API ni connexion, et leur usage est gratuit [S24].
  - **Le contenu est chiffré de bout en bout** (RFC 8291) : le service du navigateur transporte le message sans pouvoir le lire [S24].
  - **Apple** : le push web marche sur iPhone depuis iOS 16.4, et il n'est pas nécessaire d'être membre du programme développeur payant d'Apple [S25][S26]. Mais seulement pour une application web **ajoutée à l'écran d'accueil**, et la demande de permission doit suivre un geste du visiteur (toucher un bouton « s'abonner ») [S26][S27].
- **Contraintes imaginées (sans source) :**
  - Le service de transport n'est **pas choisi par l'auteur** : c'est celui du navigateur du visiteur (Google pour Chrome, Apple pour Safari, Mozilla pour Firefox). Il ne lit pas le message, mais il voit **quand** un message passe et vers quel abonné.
  - Les demandes de permission agressives fatiguent les gens, qui refusent tout.
  - Le RGPD : l'abonnement est lié à une personne ; il faut pouvoir se désabonner facilement et l'abonnement doit être supprimé avec le compte.
- **Est-ce compatible avec « aucun prestataire obligatoire » ?** Oui, à mon avis, **si c'est facultatif** : l'auteur ne signe rien chez personne, et son site marche entièrement sans. Le relais appartient au navigateur du visiteur, pas à l'auteur — exactement comme le réseau du visiteur.
- **Verdict : ouvrir sous conditions.**
- **Forme proposée en HoloCode :**
  ```
  Notify(name: nouvelles,
         ask: on-tap("Être prévenu des nouveaux messages"),
         channel: push,
         fallback: email)
  ```
  - Le moteur génère les clés VAPID dans `holo serve`, une fois, et les garde ;
  - permission demandée **seulement** après un toucher, jamais au chargement ;
  - désabonnement en un toucher, abonnement supprimé avec le compte ;
  - repli : le direct quand la page est ouverte (déjà fait), sinon l'e-mail envoyé par le serveur de l'auteur ;
  - sur iPhone, le moteur explique qu'il faut d'abord ajouter le site à l'écran d'accueil.
- **Risques de ma proposition :** l'auteur ne contrôle pas le délai ni la fiabilité de livraison ; sur iPhone, peu de gens installent les applications web.
- **Ce que je n'ai pas pu vérifier :** les conditions d'utilisation actuelles de chaque service de push (Google, Mozilla) ; je n'ai pas lu le texte complet de la RFC 8291, seulement des résumés et la RFC 8292.

---

### 8. Payer sur la page

- **Contraintes trouvées (avec leurs sources) :**
  - **PCI DSS** : si le serveur de l'auteur voit un seul numéro de carte, même sans le garder, il passe au questionnaire le plus lourd, le SAQ D : environ 300 à plus de 360 exigences [S28][S29]. Si la carte est saisie dans un cadre du prestataire et que le serveur ne la voit jamais, c'est le SAQ A, environ 22 questions [S28][S30].
  - Le coût : un guide de 2026 pour l'Afrique estime l'écart de conformité entre 500 000 et 3 000 000 FCFA par an [S30].
  - **DSP2 / authentification forte** : en Europe depuis 2019, le paiement par carte en ligne exige en général une vérification 3-D Secure [S31].
  - **L'API Payment Request** du W3C affiche seulement une fenêtre de paiement du navigateur et récupère les informations ; **c'est ensuite le marchand qui les envoie à un prestataire** pour réellement payer [S32]. Elle ne supprime donc pas le prestataire, et demande JavaScript.
  - **Mobile money** : pour M-Pesa (Daraja), Safaricom ne donne pas d'accès « production » à un particulier : il faut une entreprise enregistrée et un compte M-Pesa entreprise (Paybill ou Till) [S33].
- **Contraintes imaginées (sans source) :**
  - **Les cryptomonnaies** : prix qui bouge, paiement irréversible, lois différentes selon les pays ; risque de fraude difficile à rattraper.
  - **Le virement et le mobile money « à la main »** (le client envoie l'argent au numéro du vendeur et tape l'identifiant de la transaction, puis le vendeur confirme) : marche **sans aucun prestataire ni contrat d'API**, mais demande une vérification humaine et attire les fausses preuves de paiement.
  - Un auteur qui fabrique son propre formulaire de carte pour « faire simple » se met en danger juridique sans le savoir.
- **Verdict : ouvrir sous conditions — et ne jamais recevoir de numéro de carte sur le serveur de l'auteur.**
- **Forme proposée en HoloCode :**
  ```
  Pay(amount: cart.total, currency: "CDF",
      methods: [
        manual(mobile-money, to: "+243 …", reference: order.id),
        manual(transfer, iban: secret("IBAN")),
        provider("flutterwave")      // facultatif, réglé sur le serveur
      ])
  ```
  - `manual(...)` : marche chez soi, sans personne ; le serveur crée une commande « en attente », l'auteur confirme ;
  - `provider(...)` : un prestataire **choisi par l'auteur**, avec ses clés sur le serveur ; la carte est saisie dans **le cadre ou la page du prestataire**, jamais dans un `Input` de HoloCode ;
  - le moteur **refuse** un `Input` qui ressemble à un numéro de carte ;
  - le résultat arrive au serveur par le retour du prestataire (pas par la page), puis met à jour la commande.
- **Risques de ma proposition :** le mode manuel ne passe pas à grande échelle ; chaque prestataire demandera un petit adaptateur côté serveur.
- **Ce que je n'ai pas pu vérifier :** les conditions exactes d'accès aux API d'**Orange Money, Airtel Money et M-Pesa (Vodacom) en RDC** pour un développeur ou une petite entreprise ; si la DSP2 touche un site congolais (elle vise l'Europe, mais un client avec une carte européenne y sera soumis par sa banque).

---

## Ce qui revient partout

1. **Le moteur tient les règles, pas l'auteur.** Les huit fonctions s'ouvrent si le moteur garde la main : liste de sites permis, titre obligatoire, valeur secrète jamais lisible, carte jamais vue.
2. **Le serveur de l'auteur sert de bouclier.** Données extérieures, miniatures de vidéo, clés d'API, paiements : passer par `holo serve` protège la vie privée du visiteur et garde « chez soi d'abord ».
3. **Facultatif, toujours.** Push, prestataire de paiement, vidéos extérieures : le site doit tourner sans eux.

---

## Sources

| # | Source | Date |
|---|---|---|
| S1 | specification.website, « Clickjacking protection (frame-ancestors / X-Frame-Options) » — https://specification.website/spec/security/frame-ancestors/ | non datée |
| S2 | Squirro, « Embedding Dashboards » (sandbox bloque scripts et fenêtres) — https://docs.squirro.com/en/latest/technical/integrations/embed-dashboards.html | non datée |
| S3 | enterno.io, « Clickjacking Prevention: X-Frame-Options vs frame-ancestors » — https://enterno.io/en/articles/clickjacking-prevention | non datée |
| S4 | CookieScript, « How to add YouTube videos without cookies » — https://cookie-script.com/blog/how-to-add-youtube-videos-without-cookies/amp | non datée |
| S5 | Per Axbom, « Embed YouTube videos without cookies » — https://axbom.com/embed-youtube-videos-without-cookies/ | non datée |
| S6 | OpenReplay, « Embedding YouTube videos » (façade) — https://blog.openreplay.com/embedding-youtube-videos/ | non datée |
| S7 | W3C, Understanding WCAG 4.1.2, technique H64 (titre des cadres) — https://w3.org/TR/2008/NOTE-UNDERSTANDING-WCAG20-20081211/ensure-compat-rsv.html ; AccessLint, « Frames must have an accessible name » — https://www.accesslint.com/core/docs/rules/labels-and-names/frame-title/ | 2008 ; non datée |
| S8 | Security Scientist, « Password Policies: What the Evidence Says » (NIST SP 800-63B-4 finalisé en août 2025) — https://www.securityscientist.net/blog/password-policy-evidence-length-complexity-rotation/ | 2026 |
| S9 | Axonbuild, « NIST minimum password length » (section 3.1.1.2 de SP 800-63B-4) — https://axonbuild.com/blog/nist-minimum-password-length/ | non datée |
| S10 | Optro, « NIST password guidelines » — https://optro.ai/blog/nist-password-guidelines | non datée |
| S11 | activeMind.legal, jugement de Munich du 20 janvier 2022 (3 O 17493/20) — https://www.activemind.legal/guides/ruling-google-fonts/ | 2022 |
| S12 | The Hacker News, « German court rules websites embedding Google Fonts violates GDPR » — https://thehackernews.com/2022/01/german-court-rules-websites-embedding.html | janvier 2022 |
| S13 | decoded.legal, « Google Fonts, an IP address, and the GDPR » — https://decoded.legal/blog/2022/02/google-fonts-an-ip-address-and-the-gdpr-must-i-now-self-host-all-my-web-page-resources | février 2022 |
| S14 | W3C, « Understanding SC 2.3.3 Animation from Interactions » — https://w3c.github.io/wcag/understanding/animation-from-interactions.html | non datée |
| S15 | Disability World, « WCAG 2.3.3 » (réglage dans les systèmes) — https://www.disabilityworld.org/toolkit/standards/wcag/2-3-3-animation-from-interactions/ | non datée |
| S16 | Sophos, sur event-stream et Copay — https://news.sophos.com/?p=432939 | novembre 2018 |
| S17 | Microsoft DevOps Blog, « Blocking malicious versions of event-stream and flatmap-stream » — https://devblogs.microsoft.com/devops/?p=56462 | novembre 2018 |
| S18 | Sonatype, « Popular npm Project Used by Millions Hijacked » (ua-parser-js) — https://www.sonatype.com/blog/npm-project-used-by-millions-hijacked-in-supply-chain-attack | octobre 2021 |
| S19 | GitHub, liste « awful-oss-incidents » — https://github.com/PayDevs/awful-oss-incidents | non datée |
| S20 | NSFOCUS, « XZ-Utils Supply Chain Backdoor (CVE-2024-3094) » — https://nsfocusglobal.com/?p=28714 | 7 avril 2024 |
| S21 | MDN, « Using Pointer Events » — https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events/Using_Pointer_Events | non datée |
| S22 | W3C, Pointer Events Recommendation — https://w3.org/TR/2015/REC-pointerevents-20150224 | février 2015 |
| S23 | IETF, RFC 8292 (VAPID) — https://datatracker.ietf.org/doc/html/rfc8292 | novembre 2017 |
| S24 | Thinktecture, « HTTP Web Push » — https://www.thinktecture.com/?p=5071 | non datée |
| S25 | Simon Willison, « Web Push for Web Apps on iOS and iPadOS » — https://feeds.simonwillison.net/2023/Feb/17/web-push-for-web-apps-on-ios-and-ipados/ | 17 février 2023 |
| S26 | GitHub humhub, citation du blog WebKit — https://github.com/humhub/fcm-push/issues/17 ; blog WebKit original : https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/ | 2023 |
| S27 | 9to5Mac, « iOS 16.4 adds new capabilities for web apps » — https://9to5mac.com/2023/02/16/iphone-web-app-new-features-ios-16-4/ | 16 février 2023 |
| S28 | Paytia, « SAQ A vs SAQ D » — https://www.paytia.com/resources/blog/saq-a-vs-saq-d | 29 mai 2026 |
| S29 | cside, « What is SAQ D » — https://cside.com/blog/what-is-saq-d-and-how-to-complete-it | non datée |
| S30 | Kolonell, « PCI-DSS compliance for card checkout in Nairobi » — https://kolonell.com/en/blog/pci-dss-compliance-card-checkout-nairobi-2026 | 2026 |
| S31 | Payroc / BlueChip, « 3D Secure and Strong Customer Authentication » — https://bluechip.payconex.net/selfcare/api_specification/3d_secure_payroc | non datée |
| S32 | web.dev, « Payment Request API overview » — https://web.dev/articles/how-payment-request-api-works | non datée |
| S33 | Mctaba, « Going Live on Daraja: The Production Checklist » — https://mctaba.com/learn/mpesa/daraja-production-checklist | non datée |

**Prudence sur les sources :** plusieurs sont des blogs ou des guides commerciaux, pas des textes officiels. Pour les décisions importantes (mots de passe, paiement), il vaut mieux relire le texte officiel : NIST SP 800-63B-4, le PCI SSC, et les conditions de chaque opérateur de mobile money.
