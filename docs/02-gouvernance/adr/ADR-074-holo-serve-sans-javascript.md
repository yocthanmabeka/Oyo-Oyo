# ADR-074 — Lot 5 du web, premier pas : `holo serve`, un serveur en Rust, et des boutons qui marchent sans JavaScript

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, lot 5) ; la proposition du serveur (`proposals/Claude/serveur-et-comptes-2026-10-07.md`) ; Holoverse de 1970 à 2026, la proposition de Codex (PR 153) et la réponse de Gemini (`docs/05-discussions/reponses/2026-10-07-gemini-metavers-tous-materiels.md`)
- Validation : Yocthan, le 2026-10-07, à la proposition « ajouter au lot 5 la condition : la boutique marche avec JavaScript coupé » : « Oui, vas-y et continue » ; vérifié dans Chrome, JavaScript coupé, avant la fusion
- Projets affectés : HoloEngine, le serveur

## Décision

1. **`holo serve [dossier] [port]`** : le moteur en ligne de commande sert un site, sur le PC de l'auteur. Un seul programme en Rust : ni Node.js, ni base à installer à part, ni service extérieur (chez soi d'abord). Il sert les pages `.holo` déjà fabriquées, les images et fichiers du dossier, et le moteur pour le navigateur.
2. **La base est un fichier SQLite**, `holo-data/site.sqlite`, dans le dossier du site, compilée avec le moteur. Ce dossier n'est jamais servi, et n'est pas versionné.
3. **Chaque visiteur a un numéro**, tiré au hasard par le système (128 bits), dans un cookie `HttpOnly`, `SameSite=Lax`. Le serveur range sous ce numéro les valeurs de chaque page (`State`). Un visiteur absent trente jours est oublié. Rien n'est partagé entre visiteurs : ce sera le lot 6. Personne n'a de compte : le lot 7.
4. **Les boutons marchent sans JavaScript.** La page fabriquée par `holo serve` garde exactement son HTML ; un formulaire caché, `holo-gestures`, en tête de page, reçoit les boutons nommés (`Button`, `Shape`) et les champs liés à une valeur, par l'attribut `form`. Sans JavaScript, toucher « Ajouter » envoie les champs et le geste (`Ajouter.tap`, ou `Fait.tap@2` dans la ligne d'une liste). Le serveur fait tourner **le même arbitre** que le navigateur : les champs d'abord, puis le toucher. Il range le nouvel état, puis renvoie la page à jour par une nouvelle demande (`303`) : recharger ne rejoue pas le geste. La touche Entrée dans un champ envoie les champs sans toucher aucun bouton.
5. **Avec JavaScript, rien ne change** : la page légère empêche ce formulaire de partir, et le moteur fait le travail dans le navigateur. Il repart des valeurs que le serveur a gardées (`data-visit`).
6. **Ce que le serveur refuse** : un geste qui n'est pas un toucher (une horloge, un survol, un signal inventé) ; un geste venu d'un autre site (`Origin`) ; un formulaire de plus de 64 Ko ; la base, un fichier caché, une sortie du dossier.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le serveur | garder Node.js ; **Rust, dans le moteur** | le même arbitre que le navigateur, sans le traduire ; un seul programme à installer |
| Recevoir les demandes | écrire le protocole HTTP à la main ; un serveur asynchrone (hyper, axum) ; **`tiny_http`** | petit, sans asynchrone, assez pour un site sur son PC ; un serveur plus lourd attendra le direct à plusieurs (lot 6) |
| Les gestes sans JavaScript | un formulaire autour de chaque bouton ; **un seul formulaire caché, rattaché par `form=`** | la mise en page ne bouge pas : le même HTML avec ou sans JavaScript |
| L'état d'un visiteur | dans la page (un champ caché) ; **dans la base, sous un numéro** | le visiteur ne peut pas écrire l'état qu'il veut ; le même chemin servira aux valeurs partagées et aux comptes |

## Ce qui n'est pas fait

- Les formulaires `Form` (les messages, les fichiers envoyés) passent encore par `outils/server.mjs` ; `holo serve` les refuse (`415`) pour l'instant.
- Ce qui demande le moteur ne marche pas sans JavaScript : les horloges (`Every`, `After`), le clavier, le survol, le glissement, les points et les mondes, les modules.
- La compression Brotli, l'éditeur et la pile restent dans `outils/server.mjs`.
- Les sauvegardes de la base, les adresses comme `/profil/123` : la suite du lot 5.

## Critères de validation

- Tests du moteur : la page d'un visiteur (boutons, ligne d'une liste, champs, case à cocher, bouton rond ; un point et un `Form` laissés tels quels) ; un formulaire lu (`+`, `%C3%A9`, nom répété) ; seuls les touchers passent ; deux touchers gardés, un autre visiteur à zéro, la base relue par un nouveau serveur ; refusés : un survol, un autre site, du JSON, un corps trop lourd, une page absente, `DELETE`, la base, un fichier caché, `..`.
- Dans Chrome, JavaScript coupé (`node outils/browser-tests.mjs serve`) : leçon 68, une tâche ajoutée avec le texte écrit, puis faite ; la touche Entrée n'ajoute rien ; leçon 14, deux pommes, « 2 fruits, 4 euros » calculés par le serveur ; JavaScript rallumé, le moteur repart de ces valeurs, et le geste reste dans la page.
