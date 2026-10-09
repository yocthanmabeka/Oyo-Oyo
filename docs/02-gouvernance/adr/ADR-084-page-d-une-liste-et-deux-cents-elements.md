# ADR-084 — Une page d'une liste : `Filter(offset:)`, et des listes de deux cents éléments

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09 : intégrer le travail de Codex quand il est bien fait)
- Date : 2026-10-08 (la construction, par Codex) ; 2026-10-09 (la relecture, l'intégration et la validation)
- Responsable : Yocthan Mabeka
- Discussions sources : la demande de Yocthan « Finis la partie web » ; les dix parcours du plan (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`) ; la PR 201 de Codex et son compte rendu (`proposals/GPT5.6/web-viable-2026-10-08/README.md`) ; les listes (`ADR-051`), les listes calculées (`ADR-062`), les clés et le total (`ADR-065`).
- Validation : Yocthan, le 2026-10-09.
- Projets affectés : HoloCode, HoloEngine
- Auteur de la construction : **Codex** (PR 201) ; relu, intégré et documenté par Claude.

## Contexte

- Une liste calculée savait chercher, filtrer, trier et couper (`limit:`, « montrer plus »), pas montrer **la page suivante** : un catalogue ne se parcourait que depuis le début.
- Une liste gardait cent éléments au plus, une répétition (`Repeat(items:)`) deux cents : un catalogue de deux cents produits ne tenait pas dans une liste.

## Décision

1. **`offset:` dans `Filter`** : combien d'éléments sauter, **après** la recherche, le filtre et le tri, **avant** `limit:`. `Filter(name: page, from: products, sortBy: price, offset: start, limit: 20, total: matching)` montre vingt produits à partir du rang `start` ; `start.add(20)` mène à la page suivante. Sans `offset`, rien n'est sauté.
2. **Le total compte avant les deux coupes** : `total: matching` dit combien la recherche trouve, pas combien la page en montre (« 20 sur 200 »). Une position au-delà de la fin donne une page vide, jamais une erreur.
3. `offset` prend un entier positif ou nul, ou le nom d'un nombre entier de la page. Un grand nombre est borné à la longueur réelle de la liste avant d'être converti : aucune allocation en proportion, aucune troncature sur un navigateur 32 bits.
4. **Deux cents éléments par liste**, comme une répétition : une liste de `State`, une liste reçue par `Data` (les deux cents premiers sont gardés), une liste calculée. Un fichier qui en déclare plus est refusé, avec la raison. Les autres bornes ne changent pas : deux cents caractères par champ, seize champs par élément, 64 Ko de données reçues.
5. Un catalogue de démonstration de deux cents produits, la leçon 109 et le site des parcours (`exemples/parcours/`) : le catalogue, une fiche et son panier (une TVA fictive calculée en décimaux exacts), une inscription, un contact avec une image, un menu, une réservation en direct, un compte, un profil, un article et une vidéo sous-titrée. Neuf des dix parcours du plan ; le dixième (un tableau de bord) vient du lot 9.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le mot | `page:` (un numéro de page), `skip:`, `start:`, **`offset:`** | `offset` est le mot de SQL (`OFFSET`), des API et de la plupart des cadres ; il dit le décalage sans supposer une taille de page ; une page de vingt s'écrit `start.add(20)`, sans calcul caché |
| Quand couper | avant le tri ; **après la recherche, le filtre et le tri** | sinon la page 2 mélange des éléments de la page 1 après un tri (le défaut classique des pages calculées avant le tri) |
| Le total | après la coupe ; **avant** | « 20 sur 200 » : le visiteur sait combien il en reste ; la leçon 109 garde deux nombres, `offset` et `end`, et cache « précédente » et « suivante » avec `If(offset, over: 0, …)` et `If(end, under: matching, …)` |
| La borne | cent ; **deux cents** ; sans borne | deux cents suffisent à un petit catalogue (environ 22 Ko de JSON) et restent rapides sur un téléphone ; au-delà, il faudra paginer sur le serveur |

## Ce qui est refusé, et pourquoi

- `offset: "1"` (un texte), `offset: 1.5` (un nombre à virgule), `offset: 1px` (une unité), `offset: true`, un nom inconnu, le nom d'un nombre à virgule : une position est un entier.
- Une liste déclarée de plus de deux cents éléments.

## Les défauts du web évités

- **La page calculée avant le tri** (un `LIMIT … OFFSET` posé avant `ORDER BY`, ou un `slice` avant `sort`) : ici, toujours après.
- **Le total compté après la coupe**, qui fait croire qu'il n'y a qu'une page : ici, avant.
- **Un grand décalage qui fait tomber la page** (une erreur, ou une allocation géante) : ici, une page vide.

## Dettes

- Une nouvelle recherche depuis une page avancée ne revient pas seule au début : la leçon et le catalogue offrent « Revenir au début des résultats ».
- Pas de pagination par le serveur : le navigateur reçoit les deux cents enregistrements, puis les découpe.
- Les essais des parcours sont dans le dossier de Codex (`proposals/GPT5.6/web-viable-2026-10-08/browser-tests.mjs`), branchés sur la suite ; à ranger dans `moteur/outils/` un jour. Leur audit d'accessibilité lit la copie locale d'axe-core (`npm install --no-save axe-core@4.10.3`, dans `moteur/`), comme `outils/accessibility.mjs` ; la première version la demandait à unpkg.com.
- Ce que les parcours ne prouvent pas (le compte rendu de Codex le dit) : un téléphone réel, une personne au TalkBack, la batterie.

## Critères de validation

- Tests du moteur : `two_hundred_products_are_searched_sorted_and_paged` (dix pages de vingt, sans doublon ni oubli après le tri ; le produit 200 retrouvé ; une position au-delà de la fin ; deux cent un éléments refusés) ; `a_page_offset_refuses_text_decimals_units_and_unknown_names`.
- Dans Chrome (`node outils/browser-tests.mjs parcours`) : les parcours 1 à 9 de Codex, avec `holo serve` (le catalogue dans le HTML du serveur, les pages au clavier et sans JavaScript, le panier exact avec et sans JavaScript, les erreurs reliées, l'image aux mêmes octets, le menu à 360 et 1280 pixels, la réservation vue dans un second profil, le compte retrouvé, le profil, la vidéo, l'impression) ; l'audit axe-core, neuf pages en quatre modes.
- Leçon `109-un-catalogue-page-par-page.holo`.
