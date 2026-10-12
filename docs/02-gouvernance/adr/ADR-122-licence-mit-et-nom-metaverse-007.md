# ADR-122 — La licence MIT, le nom `Metaverse_007`, et la candidature au programme Claude for Open Source

- Statut : ACCEPTÉ (Yocthan, 2026-10-12 : « Passer en MIT » ; « Le dépôt et tous les textes » ; « Aujourd'hui on ne code rien, dispose juste le projet pour la candidature »)
- Date : 2026-10-12
- Responsable : Yocthan Mabeka
- Discussions sources : la demande de Yocthan du 2026-10-12 (préparer la candidature d'Oyo-Oyo au programme *Claude for Open Source* d'Anthropic) ; le fichier `LICENSE` du 2026-10-08 (« tous droits réservés ») ; le journal du 2026-10-08 (le dépôt devient public, sous le nom `Oyo-Oyo`) ; `ADR-060` (le code en anglais, les textes en français).
- Validation : faite par Yocthan le 2026-10-12, après que les conséquences lui ont été dites (voir « Ce que cela change »).
- Projets affectés : tout le dépôt.

## Contexte

- Le programme *Claude for Open Source* d'Anthropic offre un abonnement aux responsables de projets libres. Il s'adresse à des projets publiés sous une licence libre, avec les repères d'usage : une licence, un guide de contribution, un code de conduite, un README lisible en anglais.
- Le dépôt était public depuis le 2026-10-08, mais sans licence libre : « Copyright (c) 2026 Yocthan Mabeka. Tous droits réservés », en français et en anglais. On pouvait le lire, pas le copier.
- Il s'appelait `Metaverse` jusqu'au 2026-10-08, puis `Oyo-Oyo`.
- Le `README` était en français seulement, et son « État actuel » décrivait encore le prototype Python de septembre 2026.

## Décision

1. **La licence MIT**, à partir du 2026-10-12, au nom de Yocthan Mabeka, pour tout le dépôt : le code du moteur, le langage HoloCode, les leçons, les documents, les images. Le fichier `LICENSE` ne contient que le texte de la licence MIT, pour que GitHub la reconnaisse. Les polices gardent leur propre licence (SIL Open Font License 1.1 : Carlito dans `exemples/lecons/`, les polices du moteur dans `moteur/web/fonts/`), ce que le `README` dit.
2. **Le nom `Metaverse_007`** : le dépôt GitHub devient `yocthanmabeka/Metaverse_007` ; GitHub fait rediriger les anciennes adresses (`Oyo-Oyo`, `Metaverse`). Le nom est employé dans le `README`, `CONTRIBUTING.md`, `AGENTS.md`, `GEMINI.md`, la politique de synchronisation et la candidature. Les adresses écrites dans le code (le `User-Agent` de `holo serve`, `moteur/src/remote.rs`) et dans la leçon 141 (un module chargé depuis GitHub) gardent l'ancien nom pour l'instant : elles continuent de marcher par la redirection, et aucun code n'est touché ce jour-là.
3. **Les repères d'un projet libre**, ajoutés sans réécrire ce qui existe : `CODE_OF_CONDUCT.md` (Contributor Covenant 2.1, le texte anglais fait foi, un préambule en français), `CONTRIBUTING.md` complété par la façon de lancer les essais du moteur et de proposer une pull request, en français puis en anglais, un « Quick start » en anglais en haut du `README`, `README.en.md` (la traduction entière, reliée dans les deux sens), une feuille de route courte avec l'objectif 2030, l'étiquette `good first issue` sur deux issues existantes (#275, #184).
4. **La candidature** : ses textes (le résumé de 150 mots, la portée et l'impact, l'usage de l'abonnement, les autres informations) sont dans `docs/05-discussions/candidatures/2026-10-12-claude-for-open-source.md`, en anglais, avec les chiffres du dépôt tels qu'ils sont ce jour-là, sans en inventer.

## Ce que cela change

- Quiconque peut copier, modifier, redistribuer et vendre le code et le langage, à condition de garder le nom de Yocthan Mabeka et le texte de la licence. **Cela ne se reprend pas** pour les copies déjà faites : une licence MIT accordée reste accordée.
- Rien ne change pour les règles du projet : on discute avant de construire, on ne fusionne que ce qui marche, seul Yocthan valide une décision. Une licence libre dit ce que les autres peuvent faire du code, pas qui décide dans ce dépôt.
- Le `README` dit « référentiel public » au lieu de « privé », et son « État actuel » renvoie à `AGENTS.md` pour l'état réel ; son ancien texte reste en place, daté.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| La licence | garder « tous droits réservés » ; **MIT** ; Apache 2.0 | MIT est la plus courte et la plus connue, celle que Yocthan a nommée ; Apache 2.0 ajoute une clause sur les brevets et l'obligation de signaler les modifications, plus protectrice mais plus longue ; garder « tous droits réservés » aurait sans doute fait refuser la candidature |
| Le fichier `LICENSE` | le texte MIT plus la note sur les polices ; **le texte MIT seul** | GitHub reconnaît une licence quand le fichier est le texte de la licence ; la note sur les polices va dans le `README` |
| Le nom | seulement dans la candidature ; la candidature et le `README` ; **le dépôt et tous les textes** | Yocthan veut que `Metaverse_007` soit le nom du projet ; GitHub redirige les anciennes adresses, les liens déjà écrits ne cassent pas |
| Les adresses dans le code et la leçon 141 | les changer le même jour ; **les laisser, redirigées** | « aujourd'hui on ne code rien » ; une adresse changée dans la leçon 141 avant le renommage aurait cassé son essai |
| Les `good first issue` | créer des issues depuis les petites dettes des ADR ; **étiqueter les deux issues existantes qui s'y prêtent** | Yocthan : « on ne code rien, dispose juste le projet » ; seul lui ajoute une tâche à la file |

## Critères de réexamen

- Si la candidature demande une autre licence ou un autre repère, on en reparle avec Yocthan ; la licence MIT, elle, reste.
- Les adresses du code et de la leçon 141 passent au nouveau nom à la prochaine PR qui touche ces fichiers.
