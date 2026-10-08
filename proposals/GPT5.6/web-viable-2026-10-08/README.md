# Finir la partie web : catalogue et recette
- Auteur : Codex, le 2026-10-08.
- Statut proposé : **EXPÉRIMENTATION** ; aucun statut de décision modifié.
- Source : demande de Yocthan « Finis la partie web », reprise de la session PC ; tâches #188 et #200 ; définition des dix parcours dans `proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`. Aucun numéro HC distinct identifié pour cette reprise.
- Décisions concernées : ADR-051 (listes), ADR-062 (listes calculées), ADR-065 (clés), ADR-069 (disposition), ADR-074 à ADR-081 (serveur, partage, comptes).
- Branche propre, au-dessus de la correction #199 ; aucune fusion dans main ou dans une branche de Claude.

## Ce que cette contribution construit
Une liste dynamique accepte 200 éléments, comme une répétition statique. Les limites de 200 caractères par champ, de 16 champs par élément et de 64 Ko par réponse de données sont conservées. Les données reçues au-delà du nombre maximal conservent le comportement existant : les premiers éléments seulement sont repris ; une déclaration .holo de 201 éléments est refusée.

`Filter(offset: position, limit: 20)` saute des résultats **après** recherche, filtrage et tri ; le total est calculé **avant** les deux coupes. L'absence de offset vaut zéro. Une position au-delà de la fin donne une page vide. Le nombre est borné à la longueur réelle avant la conversion en usize, donc sans troncature sur wasm32 et sans allocation en proportion d'un grand offset. L'écriture « offset » garde le sens usuel de décalage ; elle est proposée, pas décidée.

Leçon 109 et site `exemples/parcours/` : catalogue JSON de 200 produits, panier, inscription, contact avec fichier, menu, réservation, profil, article et média local. Le profil utilise un titre de partage fixe ; le titre dynamique attend le correctif du nuage #193. Le panier montre une unité vendue 12,50 € HT et une TVA fictive de 20 % : Le moteur additionne le prix HT, multiplie le total par 1,20 et calcule la différence : quatre unités donnent **50,00 € HT + 10,00 € TVA = 60,00 € TTC**, en décimaux exacts. C'est une recette arithmétique, pas un moteur fiscal.

## Recette ajoutée
Le module `browser-tests.mjs` de ce dossier s'insère dans la suite Chrome existante. Il copie le site dans un dossier temporaire et utilise le vrai binaire Rust en release ; aucun compte réel ni base du dépôt touchés. Une petite image PNG sert au téléchargement de fichier. La bibliothèque axe-core 4.10.3 est chargée seulement pour l'audit, depuis son paquet publié ; aucun service extérieur n'est nécessaire au site ou au serveur.

| Parcours | Ce que l'automate vérifie | Ce qu'il ne prouve pas |
|---|---|---|
| 1 | 200 données ; HTML du serveur ; pages, recherche sur le produit 200, catégorie et tri ; activation au clavier ; sans JavaScript | pertinence d'un classement réel dans un moteur de recherche |
| 2 | ajout/retrait et total TTC exact, JavaScript activé et coupé | commande commerciale, taxes de plusieurs pays, paiement |
| 3 | erreurs reliées aux champs, inscription valide et message enregistré, envoi unique | livraison d'un e-mail |
| 4 | image choisie dans Chrome, formulaire envoyé, octets conservés | essai humain du sélecteur de fichiers ; envoi du fichier sans JavaScript |
| 5 | mêmes actions à 360 et 1280 pixels, menu au clavier, pas de débordement | téléphone matériel, confort visuel humain |
| 6 | deux pages et SSE : réserver ici, voir ailleurs sans recharger | identité unique d'un visiteur anonyme, contrôle du débit |
| 7 | un compte et le panier après effacement des cookies, reconnexion | deux appareils physiques |
| 8 | adresse /profil/123, titre visible, description et image de partage, moteur chargé ensuite | aperçu effectivement utilisé par les réseaux sociaux |
| 9 | texte, sous-titres reçus, lecture vidéo, impression sans menu, média local | qualité des sous-titres entendus/lus par une personne |
| 10 | laissé au lot 9 du nuage | pas revendiqué comme fini |

L'audit automatique contrôle les pages à 360 et 1280 pixels, en clair et en sombre. Il ne remplace pas une personne au TalkBack. Les boutons éprouvés au clavier sont activés par Tab et Entrée ; le choix d'un fichier est piloté par DevTools.

## Résultats
Code et essais préparés ; résultats à inscrire après exécution et lecture des journaux de GitHub Actions. Aucun résultat supposé n'est présenté comme obtenu.
Le terminal local ne démarre pas ; aucun essai local ou sur le Samsung n'est revendiqué.

## Limites et objections
- Augmenter la limite à 200 double la borne maximale du nombre d'éléments ; les pages doivent garder une limite de rendu raisonnable. Le site de recette n'affiche que 20 cartes. Pas de mesure de mémoire ou batterie par cette contribution.
- Les données du catalogue sont une photographie locale. Pas d'index de base de données ni de pagination réseau : le navigateur reçoit les 200 petits enregistrements, environ 21 Ko JSON.
- Une recherche depuis une page avancée exige « Revenir au début des résultats » ; aucun événement nouveau de champ n'est introduit.
- Les comptes gardent leurs dettes #180/#181 ; le partage garde les siennes #182. Les capacités du nuage #189 ne sont pas modifiées.
- Les tests d'accessibilité demandent une connexion Internet pour télécharger la bibliothèque de test. Le site lui-même fonctionne sans cette bibliothèque.

## Ce qui reste pour déclarer le web fini
1. Intégrer, après relecture par Claude PC, #199 et cette PR avec les comptes #177.
2. Finir/revoir les tâches comptes et partage #180 à #182 ; les nouvelles dépendances des clés d'accès attendent l'accord explicite prévu par #181.
3. Relire et intégrer les PR du lot 9 du nuage ; vérifier ce qui reste de l'appareil, du hors-ligne, des imports/exports et notifications, sans confondre une chaîne de PR avec tout le lot.
4. Exécuter le parcours 10 avec ces capacités ; compléter le fichier sans-JavaScript si exigé.
5. Faire les dix parcours au clavier et au TalkBack avec une personne, puis sur le vrai Samsung ; mesurer la durée, la mémoire, la chaleur et la batterie.
6. Mettre à jour le grand tableau et le bilan seulement d'après ces preuves.

## Documents à mettre à jour par Claude après validation
Le guide (offset et borne 200), NOMS, TABLEAU-WEB, les ADR de listes si Yocthan le décide, l'index des leçons (109), le journal et le tableau de passation. Ces documents transversaux ne sont pas modifiés ici.
