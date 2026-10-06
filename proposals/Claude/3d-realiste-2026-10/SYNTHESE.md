# Synthèse : la 3D réaliste qui croît petit à petit

- Auteur : Claude
- Date : 2026-10-06
- Statut : **PROPOSITION**, à valider par Yocthan. Rien n'est construit.
- Sources : le prompt (`docs/05-discussions/prompts/2026-10-06-*-3d-realiste.md`) ; la réponse de Gemini (`docs/05-discussions/reponses/2026-10-06-gemini-3d-realiste.md`) ; l'avis de ChatGPT sur Gemini (`docs/05-discussions/reponses/2026-10-06-chatgpt-sur-gemini-3d-realiste.md`) ; la proposition de Codex (PR 118, `proposals/GPT5.6/3d-realiste-evolutive-2026-10-06/README.md`) ; la revue de Codex du 2026-10-04.

## La question de Yocthan

« Ultra réaliste si possible, mais en commençant bas et en croissant petit à petit, avec des extensions s'il le faut. Possible, ou à repenser ? »

**Réponse des trois : possible, sans revoir la vision.** Personne ne dit le contraire.

## Ce sur quoi tout le monde est d'accord

| Point | Claude | Gemini | ChatGPT | Codex |
|---|---|---|---|---|
| Tout ce qui est lourd se prépare à l'avance, sur PC ; le téléphone ne fait que lire | oui | oui | oui | oui |
| Objets proches pleins (triangles) ; les points gardés pour le lointain et pour « entrer » | oui | oui | oui | oui |
| Le tampon de profondeur d'abord | oui | oui | oui | oui |
| Couleurs calculées en linéaire, rendu des couleurs à la fin | oui | oui | oui | oui |
| Éclairage par une image du ciel, occlusion précalculée, matériaux de glTF | oui | oui | oui | oui |
| Textures KTX2 avec mipmaps | à mesurer | oui | oui | oui |
| Un module enfermé calcule, ne touche jamais au GPU | — | oui | oui | oui |
| Pas de pont JavaScript | oui | oui | — | oui |
| La chaise de Poly Haven (CC0) comme premier essai | oui | oui | oui | oui (Rockingchair 01) |

## Ce qui a changé grâce à eux

1. **Le principe** (ChatGPT, repris par Codex) : *le fichier HoloCode décrit une qualité visuelle souhaitée, jamais une technique graphique ; toute technique reste remplaçable par le moteur.* C'est ce qui permet de croître sans réécrire : le lancer de rayons, les splats ou une technique de 2030 deviennent de nouveaux moteurs de dessin, pas de nouveaux mots.
2. **Les paliers bougent pendant la visite** (ChatGPT, Codex) : un palier de départ choisi selon l'appareil, puis le moteur descend ou remonte selon le temps d'image réel. Codex propose : descendre si le 95e centile dépasse 20 ms pendant 2 à 3 secondes ; remonter après 10 secondes stables ; un seul réglage à la fois ; baisser d'abord la résolution et les ombres, pas les formes proches ; 30 images par seconde stables plutôt que 45 à 60 instables au palier léger.
3. **Un seul moteur, plusieurs représentations du même objet** (Codex, ChatGPT) : une chaise est une identité ; son fichier contient plusieurs niveaux de détail, et le moteur choisit. Pas de `weight:` dans le langage.
4. **Rien d'interdit pour toujours** : lancer de rayons et splats ne sont pas des dépendances, mais restent possibles comme paliers futurs.
5. **Le chiffre de Gemini sur la mémoire est retiré** (350–450 Mo, sans source). Nos propres cibles, plus prudentes : environ 160 Mo au palier léger, 220 Mo au palier normal, abandon au-delà de 300 Mo (mémoire propre de l'onglet, PSS) ; à mesurer.

## Où ils divergent, et l'avis de Claude

- **Le découpage du moteur** : Gemini veut une interface générique pour chaque étape du dessin ; Codex préfère des étapes écrites à la main (`OpaquePass`, `PointsPass`, `PostProcessPass`), et généraliser après deux ou trois. **Claude suit Codex** : moins d'abstraction avant le besoin.
- **Les extensions de rendu** : Gemini propose des parties du moteur exclues à la compilation ; Codex des parties officielles du moteur, activées si l'appareil le permet. Un seul moteur est servi à tous : **Claude propose de les charger à la demande**, comme `page-moteur.js` (`ADR-033`).
- **Les mots du langage** : Codex propose `Material(name:, kind: wood, finish: waxed)`, `Model(source:, material:)`, `Light(kind: sun, mood: sunset)`. **Claude propose d'attendre la chaise** : pour le premier essai, `Model(source: "chair.holo3d")` suffit, le fichier connaît déjà sa matière, et un ciel par défaut éclaire. Les mots `Material` et `Light` viendront quand un vrai besoin apparaîtra, selon la règle de Yocthan : chercher la forme la plus courte.
- **Les budgets de la chaise** : Gemini (150 Ko au palier léger) est plus serré que Codex (300 Ko). Ce sont des paramètres d'essai ; on garde ceux de Codex, plus réalistes pour un objet vu de près.

## Ce que Yocthan a à décider

1. **Le principe**, en `ADR-048` : HoloCode décrit une qualité visuelle, jamais une technique ; toute technique reste remplaçable.
2. **La direction**, en `ADR-049` : objets pleins préparés à l'avance (`.holo3d`, versionné, borné, plusieurs niveaux de détail), points pour le lointain, paliers qui bougent pendant la visite, conventions dès le premier jour (1 unité = 1 mètre, axes de glTF, couleurs linéaires).
3. **Le premier chantier**, s'il dit oui : le tampon de profondeur, le dessin des objets pleins séparé de celui des points, les conventions, et un premier objet plein sans texture, en WebGPU et en WebGL 2. Puis l'outil de préparation, le matériau, KTX2, le ciel, et la chaise mesurée sur le Flip 3.

Les mesures restent à faire : aucun des chiffres de cette synthèse n'a été mesuré pour la 3D (Codex n'a pas pu lancer le code ; Claude non plus, depuis le nuage, sur un téléphone).
