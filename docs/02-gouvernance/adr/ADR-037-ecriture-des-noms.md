# ADR-037 — L'écriture des noms : celle de Flutter

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : prompt et réponses du 2026-10-06 (`docs/05-discussions/prompts/2026-10-06-majuscules-et-casse.md`, `docs/05-discussions/reponses/2026-10-06-majuscules-et-casse.md`)
- Validation : décidé par Yocthan le 2026-10-06. À la question « garder `_` ou l'écriture de Flutter ? », il a répondu : « on peut garder les deux, mais Flutter, c'est la base pour moi » ; puis, entre trois façons de garder les deux, il a choisi « Flutter seul + correction ».
- Projets affectés : HoloCode, HoloEngine

## Contexte

HoloCode joignait deux mots de trois façons : `apple_x` et `top_right` (avec `_`), `font-size` (avec `-`, dans les styles), `BlueDoor` (collés). ChatGPT, Gemini et Claude étaient d'accord sur tout le reste (respecter la casse, refuser une faute avec le bon mot, garder `KB`), et en désaccord ici : Gemini gardait `_` (plus facile sur un téléphone), ChatGPT proposait l'écriture de Flutter.

## Décision

1. **La casse compte**, et chaque mot a **une seule** écriture.
2. **L'écriture de Flutter** : un bloc et le nom d'un bloc ont une majuscule au début et à chaque mot (`BlueDoor`, `name: AddSunrise`) ; une valeur, un paramètre, un mot-valeur ont une minuscule au début et une majuscule à chaque mot suivant (`appleX`, `topRight`). Jamais de `_`.
3. **Les styles gardent l'écriture du CSS** (`font-size`, `.carte`), comme Yocthan l'a voulu (`ADR-017`).
4. **Une faute n'est jamais avalée** : le moteur la refuse et donne le bon mot (`apple_x` → « écris `appleX` »). Un éditeur pourra plus tard la corriger d'un clic ; le moteur, lui, reste strict.
5. `KB`, `MB`, `GB` restent en majuscules : `b` voudrait dire bit.

## Comparaison faite avant de choisir

| Option | Pour | Contre |
|---|---|---|
| `_` partout (Gemini) | facile à taper sur un téléphone ; rien à changer | une écriture de plus que Flutter, que Yocthan connaît |
| **L'écriture de Flutter** (ChatGPT, choix de Yocthan) | celle que Yocthan connaît ; un signe de moins | une majuscule au milieu d'un mot, moins commode sur un clavier de téléphone |
| Accepter les deux comme un même mot (comme Nim) | souple | deux écritures du même nom dans un fichier ; critiqué chez Nim |
| Chaque fichier choisit | souple | deux mondes de fichiers ; un exemple copié d'un fichier à l'autre ne marche plus |

## Conséquences

- Les exemples et les leçons ont été convertis : `apple_x` → `appleX`, `blue_door` → `blueDoor`, `top_right` → `topRight`, etc.
- Sur un téléphone, une majuscule au milieu d'un mot coûte un geste de plus : un éditeur avec complétion compensera.

## Critères de validation

- Leçon 39 ; tests du moteur (`etat.rs`, `ce_qui_est_mal_ecrit_est_refuse_avec_un_message_clair`).
