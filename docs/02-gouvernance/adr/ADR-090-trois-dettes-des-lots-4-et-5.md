# ADR-090 — Trois dettes des lots 4 et 5 : un titre qui lit les valeurs, `narrow:` dans `Row` et `Column`, `keep` par adresse

- Statut : PROPOSITION (construit et essayé, à valider par Yocthan)
- Date : 2026-10-08
- Responsable : Yocthan Mabeka
- Discussions sources : les tâches confiées par Yocthan à la session du nuage le 2026-10-08 (`AGENTS.md`, « Trois petites dettes des lots 4 et 5 ») ; les dettes écrites dans l'`ADR-069` (`narrow:` ne mesure que les cases de `Grid`) et dans l'`ADR-078` (le titre ne lit pas `{id}` ; les valeurs gardées d'un modèle sont partagées par toutes ses adresses).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- `Page(title: "Le profil de {id}")` écrivait les accolades telles quelles dans l'onglet : le titre était le seul texte de la page qui ne lisait pas les valeurs.
- `narrow: { … }` (`ADR-069`) ne valait que dans une case de `Grid`. Une carte posée dans un `Row`, qui rétrécit avec lui, ne changeait pas.
- `keep: [pages]` dans un modèle d'adresse (`ADR-078`) rangeait les valeurs sous le nom du fichier : `/profil/ada` et `/profil/bob` partageaient les mêmes.

## Décision

1. **Le titre lit les valeurs, comme un texte** : `Page(title: "Le carnet de {nom} : {pages} page(s)")`. Un nombre prend son format (`{prix:cents}`) ou ses chiffres après la virgule, un texte s'écrit tel quel, une valeur d'adresse aussi. La page fabriquée (le serveur d'essai, `holo serve`, `holo html`) l'écrit avec les valeurs du départ, ou celles du visiteur ; quand elles changent, la page récrit l'onglet. Un titre sans accolades ne demande rien à la page.
2. **`narrow:` vaut aussi dans les cases de `Row` et de `Column` qui reçoivent une part de la place** : un bloc qui grandit (`grow:`) ou qui a une largeur en % dans un style (`.carte { width: 45%; }`), rangé dedans ou dans un `If`, un `Repeat` posé dedans, est mesuré comme une case de grille. `.carte { width: 45%; narrow: { padding: 8px; } }` se serre sur un téléphone, pas sur un ordinateur. Un bouton ou un lien qui n'a que la largeur de son texte n'est pas une case : il n'est jamais « étroit » pour autant. Rien à déclarer, comme avant.
3. **Les valeurs gardées sont rangées sous l'adresse de la page**, plus sous son fichier : un modèle garde des valeurs pour chaque adresse qu'il sert. `/profil/ada` retrouve les siennes, `/profil/bob` part de zéro. Pour une page ordinaire, l'adresse est celle de son fichier : rien ne change.

## Ce qui est refusé, et pourquoi

- Un titre qui nomme une valeur que la page n'a pas : `title: "Panier ({rien})"` est refusé, avec le même message qu'un texte (« aucune valeur ne s'appelle « rien » »).

## Les défauts du web évités

- **Le titre de l'onglet écrit à la main dans le code** (`document.title = …` dans un effet, React Helmet) : ici, le titre est un texte comme les autres, et suit ses valeurs seul.
- **Les requêtes de conteneur à déclarer** (`container-type: inline-size` sur le parent, puis `@container`) : ici, rien à déclarer, la page mesure les cases de `Grid`, de `Row` et de `Column`.
- **Une clé de stockage commune à toutes les adresses d'un même code** : ici, une par adresse.

## Dettes

- Sans JavaScript, `narrow:` ne vaut toujours pas : c'est la page qui mesure.
- Une case de `Row` qui passe seule à la ligne reçoit toute la largeur : elle n'est plus étroite. C'est voulu (c'est sa place).
- Une largeur en pixels (`width: 200px`) ne fait pas d'une case une part de la place : elle a la taille qu'on lui donne, quelle que soit la place.
- Les valeurs gardées par une page à adresse propre (`/contact` pour `contact.holo`) avant ce changement restaient sous le nom du fichier : elles ne sont pas reprises. Les deux, adresses propres et modèles, datent du même jour (`ADR-078`).

## Critères de validation

- Test du moteur : `the_title_reads_the_values_of_the_page` (un titre avec un nombre formaté, récrit pour un autre état ; un titre sans valeur, inchangé ; une valeur d'adresse accentuée ; un nom inconnu refusé).
- Dans Chrome : « un titre qui lit les valeurs, des valeurs gardées par adresse, narrow dans un Row (leçons 112 et 113) » (l'onglet d'Ada, puis récrit après deux pages ; celui de Bob ; Bob à zéro ; Ada retrouve ses deux pages ; les cartes larges sur un ordinateur, étroites sur un téléphone de 390px) ; « la mise en page » (leçons 89 à 93) garde ses trois cases étroites : les titres et les liens d'un `Column` ou d'un `Row` ne sont pas mesurés.
- Leçons `112-une-adresse-qui-se-souvient.holo` (et son modèle `112-carnets/{nom}.holo`) et `113-une-rangee-qui-se-serre.holo` ; la leçon 100 a maintenant un titre par profil.
