# ADR-046 — L'éditeur : la faute à sa place, la correction d'un clic, les mots à toucher

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : les réponses sur la casse (`docs/05-discussions/reponses/2026-10-06-majuscules-et-casse.md`) : ChatGPT, Gemini et Claude d'accord pour « corriger d'un clic dans l'éditeur, le moteur restant strict » (« il faut un éditeur : à faire ») ; `ADR-037` (« un éditeur avec complétion compensera » la majuscule au milieu d'un mot sur un téléphone) ; Yocthan, le 2026-10-06 : « Travaille sur l'éditeur now »
- Validation : validé par Yocthan le 2026-10-06, après l'avoir essayé : « valide le point 1, 2, j'ai testé et ça marche en tout cas, donc du coup il faut le valider »
- Projets affectés : outils (éditeur, extension VS Code), HoloEngine (deux fonctions), serveur d'essai

## Contexte

Le moteur refuse une faute et dit le bon mot (« écris « H1 » »), mais il fallait aller corriger à la main, ligne et colonne en tête. Et sur un téléphone, `topRight` coûte un geste de plus qu'`top_right`. Il manquait un éditeur.

## Décision

Deux éditeurs, une seule façon de corriger.

1. **L'éditeur dans le navigateur** (`/editeur`, servi par le serveur local) : sur le PC et sur le téléphone, sans rien installer.
   - Le texte en couleurs, avec ses numéros de ligne ; à côté (ou, sur un téléphone, dans l'onglet « Aperçu »), **la page telle que le visiteur la verra**, fabriquée par le moteur à chaque pause de l'écriture.
   - **La faute à sa place** : soulignée, sa ligne marquée, le message du moteur en bas ; un toucher sur le message mène à la faute.
   - **La correction d'un clic** : quand le moteur dit le bon mot, ou qu'un mot connu est tout proche (`Butten` → `Button`, `chldren` → `children`), un bouton « Remplacer « h1 » par « H1 » ». Rien n'est corrigé sans ce clic.
   - **Les mots à toucher**, en bas : les blocs dans une liste `children: [`, les réglages du bloc où l'on écrit, les valeurs du fichier après `{` dans un texte, les signaux après `Ajouter.`, les demandes après `panier.`, les réglages et les états dans un style. Tab prend le premier.
   - **Enregistrer** (`Ctrl+S`) écrit le fichier sur le PC, sous `exemples/`, avec **la clé** que le serveur tire au hasard et affiche à son démarrage ; sans la clé, on lit et on essaie, rien ne s'écrit. L'ancienne version est gardée dans `editor-backups/` (jamais versionné). Un nouveau fichier se crée depuis le panneau « Fichiers ».
2. **L'extension VS Code** (version 0.2.0) : la faute soulignée pendant qu'on écrit, même avant d'enregistrer ; l'ampoule (`Ctrl+.`) « Remplacer « h1 » par « H1 » » ; les mots du langage proposés.
3. **Une seule logique de correction**, `moteur/web/fixes.js`, copiée dans l'extension à l'empaquetage. Le moteur donne deux choses de plus : `verifier_texte` (la faute avec sa ligne, pour un texte pas encore enregistré ; en ligne de commande, `holo check -`) et `vocabulaire` (tous les mots du langage ; `holo vocabulary`).

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Quel éditeur | seulement VS Code ; seulement le navigateur ; **les deux, avec la même correction** | Le navigateur marche sur le téléphone, où Yocthan essaie tout ; VS Code est là où il écrit sur le PC, et l'extension existait déjà. |
| Qui corrige | le moteur, en silence ; **l'éditeur, sur un clic** | Le moteur reste strict (`ADR-037`) : une faute n'est jamais avalée ; l'outil est indulgent, mais visible. |
| Ce qu'on propose | le seul mot dit par le moteur ; **aussi le mot connu le plus proche** | Une faute de frappe (`Butten`) est la plus fréquente ; on ne propose que si l'écart est petit (deux ou trois lettres). |
| Le code de l'éditeur | une bibliothèque d'éditeur (CodeMirror, Monaco) ; **un champ de texte ordinaire, et ses couleurs dessous** | Rien à installer ni à télécharger ; le clavier du téléphone, l'annulation et le copier-coller restent ceux du navigateur. |
| Écrire sur le disque depuis le Wi-Fi | toujours ; jamais ; **avec une clé** | Une clé tirée au hasard à chaque démarrage : un autre appareil du réseau peut lire, pas écrire. |

## Conséquences

- Le moteur ne donne que la première faute : une fois corrigée, la suivante apparaît.
- L'aperçu est la page à son départ (sans les gestes) ; « Ouvrir la page » la montre vivante.
- L'éditeur ne coupe pas les longues lignes : on les fait défiler de côté (à revoir pour le téléphone).
- Pour l'extension, la nouvelle version doit être installée dans VS Code (une commande, dans son README) ; elle se sert du moteur construit dans le dépôt.

## Critères de validation

- Dans Chrome : « h1 » corrigé d'un clic et l'aperçu qui montre aussitôt la page ; « textx » → « text » ; les blocs proposés dans `children: [` ; l'enregistrement avec la clé (et la sauvegarde de l'ancienne version), refusé sans la clé ; les onglets sur un écran de téléphone.
- Pour l'extension, avec une imitation de VS Code et le vrai moteur : la faute soulignée à sa place, l'ampoule et son remplacement, les mots proposés. Test du moteur `l_editeur_recoit_la_faute_et_le_vocabulaire`.
