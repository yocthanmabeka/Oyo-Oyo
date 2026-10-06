# ADR-052 — La place qui reste, et un thème partagé

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md` (point 3 : « la disposition qui manque ») ; le guide, § 11 (« pas d'élément qui prend la place restante », « pas de fichier de styles à part ») ; `ADR-024` (la disposition) ; `ADR-029` (les imports)
- Validation : Yocthan, le 2026-10-06 : « je suis tes propositions, je les valide, tu as mon feu vert ».
- Projets affectés : HoloCode, HoloEngine

## Décision

1. **`grow:`** sur un bloc rangé dans `Row` ou `Column` : il prend la place qui reste, comme `Expanded` en Flutter. Un nombre entier de 1 à 12 : la part de la place qui reste. Un champ de saisie qui grandit s'étire jusqu'au bout. Ailleurs, `grow:` est refusé avec l'endroit où le mettre.
2. **La largeur d'un bloc** reste un style (`width`, `max-width`), comme avant : rien de nouveau à apprendre.
3. **Un fichier de styles seuls** est un thème : `import "theme.holo"` en haut d'une page prend tous ses styles ; la page garde les siens quand elle écrit le même. L'éditeur le reconnaît (« un fichier de styles, à importer dans une page »).
4. **Plusieurs noms de style par bloc** ont été décidés avec les composants (`ADR-050`).

## Alternatives écartées

- **Un bloc `Expanded(child: …)`** comme en Flutter : une enveloppe de plus autour de chaque bloc ; `grow: 1` dit la même chose en un réglage, comme `x:` sur un plateau.
- **`flex:` en CSS** : la disposition ne se règle pas dans un style (`ADR-017`) ; et « flex » est un jargon.
- **Un fichier `.css`** : il faudrait un second lecteur, et le CSS complet laisserait passer ce que HoloCode refuse.

## Critères de validation

- Tests : `grow` rendu, refusé hors de `Row`/`Column` et hors de 1 à 12 ; un fichier de styles importé, vérifié par l'éditeur.
- La leçon 72 dans Chrome, sur PC et sur un écran de téléphone : le champ prend la place qui reste ; une part, deux parts, sa taille ; le thème importé.
