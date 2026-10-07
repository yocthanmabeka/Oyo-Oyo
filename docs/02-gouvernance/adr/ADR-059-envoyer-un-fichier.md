# ADR-059 — Envoyer un fichier par un formulaire

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : `ADR-042` (le formulaire) ; le comparatif (`COMPARATIF-LANGAGES-WEB.md`, § 10 : « l'envoi vers un vrai serveur »)
- Validation : Yocthan, le 2026-10-07 : « L'envoi d'un fichier […] travaille aussi sur ça. »
- Projets affectés : HoloCode, HoloEngine, serveur d'essai

## Décision

1. **`Input(type: file, value: photo, label: "…", accept: image, max: 2MB)`**, dans un `Form` seulement, quatre au plus par formulaire.
2. **`accept:` est obligatoire**, avec des sortes nommées : `image` (PNG, JPEG, WebP, GIF), `pdf`, ou `[image, pdf]`. Pas de types MIME à écrire, pas d'extension.
3. **`max:`** de `1KB` à `10MB` ; `2MB` par défaut.
4. **La valeur est un texte** : le nom du fichier choisi (sans dossier, 120 caractères au plus), `""` sinon. On le montre, on le teste, une règle le vide (`photo.set("")`).
5. **La page vérifie** la sorte et la taille au moment du choix, et le dit avec le message du navigateur (en français ou en anglais, selon la langue de la page) ; un fichier refusé est retiré. Avec un fichier, l'envoi part en plusieurs morceaux (`multipart/form-data`) : les valeurs, puis les fichiers.
6. **Le serveur ne croit pas la page** : il demande au moteur (`holo fichiers page.holo`) quels champs de fichier la page a, ce qu'ils acceptent et leur taille ; il lit la sorte dans les premiers octets ; il range le fichier dans `messages/fichiers/<page>/` sous un nom tiré au hasard (le nom du visiteur n'est jamais un chemin) ; il vérifie tout avant d'écrire quoi que ce soit ; 500 Mo au plus par page. Sans moteur construit, il refuse les fichiers.

## Ce qui n'est pas fait

- Plusieurs fichiers dans un même champ, glisser-déposer, une image montrée avant l'envoi : plus tard, si un exemple le demande.
- D'autres sortes (son, vidéo, tableur) : à ajouter une à une, chacune avec sa signature.
- Un vrai serveur en ligne et des comptes : une proposition à discuter (`proposals/Claude/serveur-et-comptes-2026-10-07.md`).

## Critères de validation

- Tests : rendu (`accept`, `data-max`), le nom du fichier comme valeur, `holo fichiers`, et huit refus (hors formulaire, sans `accept`, sorte inconnue, trop grand, taille sans unité, valeur nombre, `accept` sans fichier, deux champs pour la même valeur).
- Dans Chrome : un fichier texte et un fichier trop lourd refusés tout de suite ; une vraie image envoyée, rangée, et le message écrit.
- Au serveur, sans passer par la page : un faux PNG (415), trop lourd (413), un champ inconnu ou d'un autre formulaire (400), un chemin dans le nom de fichier ignoré.
- L'audit axe-core : les 76 leçons, 0 défaut.
