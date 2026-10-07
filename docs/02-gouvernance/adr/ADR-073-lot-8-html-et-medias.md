# ADR-073 — Lot 8 du web : un encadré, des liens qui ouvrent un onglet ou téléchargent, des sous-titres, des images qui viennent en approchant, l'impression

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, lot 8) ; les réponses de Codex et de Gemini (`docs/05-discussions/reponses/2026-10-07-codex-tout-le-web.md`, `…-gemini-tout-le-web.md`) ; le grand tableau (`aside`, `a target, download`, la vidéo sans sous-titres)
- Validation : Yocthan, le 2026-10-07, en confiant le lot 8 à la session du nuage : « Vas-y, commence, c'est toi qui vois, parce qu'il faut qu'on aille rapidement » ; vérifié dans Chrome avant la fusion
- Projets affectés : HoloCode, HoloEngine

## Décision

1. **`Aside(children: [ … ])`** : un encadré à part, à côté du texte (« Le saviez-vous ? »). Le moteur écrit `<aside>`, que les lecteurs d'écran annoncent comme un contenu complémentaire. Allure de base : un trait à gauche, que les styles changent (`Aside { … }`).
2. **`A("…", to: "https://…", newTab: true)`** : le lien s'ouvre dans un nouvel onglet. Le moteur écrit `target="_blank" rel="noopener"` (la page ouverte ne peut pas toucher à celle-ci) et ajoute un texte caché, lu par le lecteur d'écran : « (s'ouvre dans un nouvel onglet) », ou « (opens in a new tab) » si la page n'est pas en français.
3. **`A("…", to: "programme.pdf", download: true)`** : le fichier se télécharge au lieu de s'ouvrir. Seulement un fichier rangé à côté : ni une page `.holo`, ni une adresse du web. Télécharger et ouvrir un nouvel onglet ensemble est refusé.
4. **`Video(…, captions: "film.vtt")`** : des sous-titres, dans un fichier WebVTT rangé à côté ; montrés d'emblée, dans la langue de la page (`srclang`), sous le nom « Sous-titres » (ou « Captions »).
5. **Les images plus bas viennent en approchant** : chaque image, sauf la première de chaque partie de la page, reçoit `loading="lazy" decoding="async"`. Rien à écrire.
6. **L'impression** : un état `print: { … }` dans un style, comme `dark:` et `phone:` ; `print: { display: none; }` cache un bloc sur papier. Le moteur cache de lui-même à l'impression ses outils (le menu ☰, le panneau des valeurs, le carrefour), les fenêtres fermées et les vidéos, et écrit l'adresse d'un lien du web à côté de son texte.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Nouvel onglet | `target: blank` comme en HTML ; **`newTab: true`** | un mot qui se lit ; `noopener` et l'annonce au lecteur d'écran viennent d'office, sans qu'on puisse les oublier |
| Les images différées | un réglage `lazy: true` ; **d'office, sauf la première** | l'auteur n'a pas à y penser ; la première image, souvent visible d'emblée, n'attend pas |
| L'impression | une feuille de style à part ; **un état `print:`** | la même écriture que `dark:` et `phone:`, déjà connue |
| Les sous-titres | plusieurs pistes, plusieurs langues ; **une piste, dans la langue de la page** | le besoin courant ; plusieurs langues viendront si un exemple le demande |

## Ce qui n'est pas fait

- **L'historique** (avant, arrière) à l'intérieur d'une page : chaque fichier `.holo` a déjà son adresse, et « retour » marche entre les pages et les mondes ; garder l'état d'une page dans son adresse viendra avec le premier vrai serveur (lot 5, des adresses comme `/profil/123`).
- Plusieurs pistes de sous-titres, une audiodescription, une image d'attente pour la vidéo.

## Critères de validation

- Tests du moteur : `Aside`, `newTab` (en anglais et en français), `download`, les sous-titres, les images différées sauf la première, `print:` ; refusés : télécharger une adresse du web ou une page `.holo`, télécharger et ouvrir un onglet ensemble, `newTab: yes`, des sous-titres en `.srt`, `Aside("texte")`, `print: { display: block; }`.
- Dans Chrome (leçons 95 et 96) : le fichier téléchargé sous son nom ; le nouvel onglet ouvert, le lien lu « … (s'ouvre dans un nouvel onglet) » ; une seule image différée ; à l'impression, le menu et la navigation cachés et l'adresse du lien écrite ; la vidéo avec sa piste « captions », en français, deux répliques.
- L'audit axe-core : 90 leçons et le site de référence, 0 défaut, en clair et en sombre sur téléphone.
