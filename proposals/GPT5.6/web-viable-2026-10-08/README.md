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
Le module `browser-tests.mjs` de ce dossier s'insère dans la suite Chrome existante. Il vérifie aussi `cargo test --release --locked`, pour contrôler les invariants avec le profil du serveur distribué. Il copie le site dans un dossier temporaire et utilise le vrai binaire Rust en release ; aucun compte réel ni base du dépôt touchés. Une petite image PNG sert au téléchargement de fichier. La bibliothèque axe-core 4.10.3 est chargée seulement pour l'audit, depuis son paquet publié ; aucun service extérieur n'est nécessaire au site ou au serveur.

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
Exécution finale sur GitHub Actions (Ubuntu, Rust 1.99, Node 22, Chrome sans fenêtre), code `3a7789dadb2fc8351fc18c07fdc73b6f52405f0b`, fusion virtuelle de test `0559fe7ade25bff61a5b3e0cf7af37388efa8515`. Aucune branche réelle fusionnée.

[Run 37789313088 : les trois tâches réussies](https://github.com/yocthanmabeka/Oyo-Oyo/actions/runs/37789313088).

| Commande ou contrôle | Résultat réellement lu |
|---|---|
| `cargo test` (debug) | 180 réussis, zéro échec, dont les deux nouveaux tests de catalogue/pagination. |
| `cargo test --release --locked` (dans la suite de recette) | 180 réussis, zéro échec ; les mêmes tests en mode optimisé. Ce ne sont pas 360 tests différents. |
| WebAssembly complet et léger en release | Les deux compilations réussies. |
| `cargo build --release --bin holo` | Réussi ; ce serveur sert réellement les parcours HTTP. |
| `node outils/browser-tests.mjs` | 48 contrôles réussis, zéro échec, 339 s : 46 scénarios Chrome et deux contrôles natifs (Rust release et fichiers .holo). |
| Ouverture des leçons dans Chrome | 102 leçons, aucune erreur. |
| `holo check` du site et de la leçon | Dix fichiers .holo acceptés ; 200 produits, 200 clés distinctes. |
| axe-core 4.10.3 | Neuf pages × quatre modes = 36 audits ; zéro défaut, aucun débordement. |
| Vérification Python de structure des cas | 22 cas (6 acceptés, 16 refusés) ; ce contrôle n'est pas à lui seul la conformité du moteur. |

Les preuves du catalogue incluent les 200 données dans le HTML du serveur, le produit 200 retrouvé, 100 livres filtrés et les dix pages triées au clavier. La recherche de catégorie est faite avec la vraie touche Flèche bas du navigateur. Le panier calcule 50,00 € HT, 10,00 € de TVA et 60,00 € TTC pour quatre unités, avec et sans JavaScript. Une image réellement jointe est conservée avec les mêmes octets. La réservation est vue sans rechargement dans un second profil Chrome. Le compte retrouve son panier après effacement des cookies et reconnexion. Chrome joue la vidéo, charge les sous-titres et produit le document d'impression.

Poids bruts WebAssembly dans ce run : **637 735 octets léger**, **3 672 928 octets complet**. Par rapport au code de base #199 : +1 697 octets léger, +1 442 octets complet. Ces chiffres ne sont ni un poids transféré compressé ni une mesure de mémoire, de fluidité, de chaleur ou de batterie.

Le [premier run 37781500002](https://github.com/yocthanmabeka/Oyo-Oyo/actions/runs/37781500002) avait 42 contrôles réussis et cinq échecs, causés par deux erreurs dans ma recette : règle `On` placée sous un `If(rules:)` où elle est refusée, et catégorie simulée avec `change` au lieu du vrai geste qui émet `input`. Le bouton de retrait a été rangé dans un `If(children:)`, sa règle reste au niveau de la page ; la catégorie est maintenant choisie au clavier. Le [second run 37782978375](https://github.com/yocthanmabeka/Oyo-Oyo/actions/runs/37782978375) a réussi ses 47 contrôles. Le troisième, ci-dessus, ajoute la vérification explicite de tous les tests Rust en release.

Vérification distincte, par lecture des journaux du nuage : [run 37720059986](https://github.com/yocthanmabeka/Oyo-Oyo/actions/runs/37720059986), branche `langage/polices-libres`, code `674caca7609713ced1ef28f61a860501fbf62d6c` : trois tâches vertes, 41 scénarios Chrome, 107 leçons, 279 s. Les modules typés, dessins, graphiques, chronomètre, historique et polices y sont éprouvés. **Cela ne prouve pas encore leur fonctionnement combiné avec les comptes et cette pagination.**

Le terminal local ne démarre toujours pas (`helper_unknown_error: setup refresh had errors`) ; aucun essai local ou sur le Samsung n'est revendiqué. Le blocage temporaire de quota GitHub a été résolu à sa reprise annoncée ; la contribution est publiée. Les résultats ci-dessus sont ceux de machines de CI.

## Limites et objections
- Augmenter la limite à 200 double la borne maximale du nombre d'éléments ; les pages doivent garder une limite de rendu raisonnable. Le site de recette n'affiche que 20 cartes. Pas de mesure de mémoire ou batterie par cette contribution.
- Les données du catalogue sont une photographie locale. Pas d'index de base de données ni de pagination réseau : le navigateur reçoit les 200 petits enregistrements, environ 21 Ko JSON.
- Une recherche depuis une page avancée exige « Revenir au début des résultats » ; aucun événement nouveau de champ n'est introduit.
- Les comptes gardent leurs dettes #180/#181 ; le partage garde les siennes #182. Les capacités du nuage #189 ne sont pas modifiées.
- Les tests d'accessibilité demandaient une connexion Internet pour télécharger la bibliothèque de test. Depuis l'intégration (PR 211), ils lisent la copie locale d'axe-core, comme `moteur/outils/accessibility.mjs` (`npm install --no-save axe-core@4.10.3`, dans `moteur/`). Le site lui-même fonctionne sans cette bibliothèque.

## Ce qui reste pour déclarer le web fini
1. Intégrer, après relecture par Claude PC, #199 et cette PR avec les comptes #177.
2. Finir/revoir les tâches comptes et partage #180 à #182 ; les nouvelles dépendances des clés d'accès attendent l'accord explicite prévu par #181. Les tâches #180 et #182 demandent d'abord la fusion de #177 ; cette fusion est réservée à Claude PC.
3. Relire et intégrer les PR du lot 9 du nuage ; vérifier ce qui reste de l'appareil, du hors-ligne, des imports/exports et notifications, sans confondre une chaîne de PR avec tout le lot.
4. Exécuter le parcours 10 avec ces capacités ; compléter le fichier sans-JavaScript si exigé. *Fait le 2026-10-09 : le tableau de bord (`dashboard.holo`, `sales.json`) et son essai, repris de la PR 204, sur `integration/parcours-10`.*
5. Faire les dix parcours au clavier et au TalkBack avec une personne, puis sur le vrai Samsung ; mesurer la durée, la mémoire, la chaleur et la batterie.
6. Mettre à jour le grand tableau et le bilan seulement d'après ces preuves.

## Documents à mettre à jour par Claude après validation
Le guide (offset et borne 200), NOMS, TABLEAU-WEB, les ADR de listes si Yocthan le décide, l'index des leçons (109), le journal et le tableau de passation. Ces documents transversaux ne sont pas modifiés ici.
