# Suite de conformité v0.1

- Statut : `PROPOSITION`
- Discussions sources : `HC-013`, revue de ChatGPT du 2026-09-21
- Décisions concernées : `ADR-007`, `ADR-008`, `ADR-009`, `ADR-014`, `ADR-015`, `ADR-016` à `ADR-020`

## À quoi sert ce dossier

Les propositions dans `proposals/` appartiennent aux IA qui les ont écrites. **Cette suite appartient au projet.** Elle dit ce que tout moteur HoloCode doit faire, quel que soit son auteur et quel que soit son langage : pour chaque fichier `.holo`, le résultat attendu est écrit à côté.

Les trois prototypes Python ne testent pas le même problème et utilisent une ancienne syntaxe ; on ne peut pas les départager proprement. Plutôt que de les comparer entre eux, on les mesurera, eux et surtout le futur moteur en Rust (`ADR-010`), contre cette suite.

Aucun moteur ne passe encore cette suite : elle précède le moteur, volontairement.

## Le format des cas

Chaque cas est une paire de fichiers dans `cas/` :

- `nom.holo` : le programme ;
- `nom.attendu.json` : ce que le moteur doit en faire.

Un cas **accepté** (`cas/valides/`) :

```json
{
  "verdict": "accepté",
  "decisions": ["ADR-009"],
  "arbre": { "bloc": "Page", "title": "Hello", "children": [ { "bloc": "P" } ] },
  "scenario": [ { "signal": "Open.tap" } ],
  "journal": [ { "entite": "Workshop", "capacite": "enter", "parce_que": ["Open.tap"] } ],
  "proprietes": ["meme-fichier-meme-resultat"]
}
```

Un cas **refusé** (`cas/refuses/`) doit être rejeté avant toute exécution :

```json
{ "verdict": "refusé", "etape": "vérification", "categorie": "budget", "ligne": 5, "decisions": ["ADR-005"] }
```

`etape` vaut `syntaxe` ou `vérification`. `ligne` est la ligne que le message d'erreur doit désigner.

### Catégories de refus

| Catégorie | Signification |
|---|---|
| `code-libre` | Du code caché dans un bloc (`ADR-015`) |
| `unite` | Une valeur n'a pas l'unité attendue |
| `budget` | Le contenu pèse plus que le budget déclaré |
| `graine` | Une graine n'est pas fixée : le résultat ne serait pas reproductible |
| `bloc-inconnu` | Un bloc qui n'existe pas |
| `capacite-inconnue` | Une capacité que l'entité n'offre pas |
| `nom-en-double` | Deux blocs portent le même nom |
| `vocabulaire` | Un mot français d'avant `ADR-016` ; le moteur indique le mot anglais à écrire |
| `casse` | Un bloc écrit sans majuscule (`h1`) ; le moteur indique l'écriture attendue (`ADR-020`) |
| `style` | Un style mal écrit : réglage inconnu, valeur incorrecte, « ; » oublié, sélecteur composé, style défini deux fois ou posé sans être défini (`ADR-017`) |
| `disposition` | Un réglage de disposition (`display`, `position`, `float`…) dans un style : un style ne dit que l'apparence (`ADR-017`) |
| `niveau-de-titre` | Un titre qui saute un niveau (`H3` après `H1`), ou un premier titre qui n'est pas `H1` (`ADR-020`) |

### Propriétés

| Propriété | Signification |
|---|---|
| `meme-fichier-meme-resultat` | Deux lectures du fichier, sur deux machines, donnent le même état et le même journal (`ADR-008`) |
| `lisible-a-plat` | Le fichier s'affiche comme une page ordinaire (`ADR-007`) |
| `visitable-en-profondeur` | Le fichier s'affiche comme un lieu où l'on zoome et où l'on entre (`ADR-007`) |
| `graines-enfants-distinctes` | Les points nés d'un morcellement ont tous des graines différentes |
| `graines-enfants-reproductibles` | Ces graines sont les mêmes d'une exécution à l'autre |
| `lisible-sans-ia` | Le monde se lit sans aucune IA (`ADR-014`) |

## La syntaxe utilisée : un brouillon

`ADR-009` fixe la forme générale (des blocs nommés par leur sens, le texte en Markdown dans les blocs) mais pas la grammaire. Les cas de cette suite utilisent le brouillon suivant, à critiquer :

```ebnf
fichier   = { import } bloc { style } ;
style     = ( NOM | "." NOM ) "{" { REGLAGE ":" VALEUR ";" } "}" ;
import    = ( "import" | "module" ) TEXTE | "bridge" ( "js" | "css" ) TEXTE ;
bloc      = NOM [ "." NOM ] "(" [ argument { "," argument } [ "," ] ] ")" ;
argument  = [ NOM ":" ] valeur ;
valeur    = bloc | liste | TEXTE | TEXTE_LONG | NOMBRE [ UNITE ] | "true" | "false" | NOM [ "." NOM ] ;
liste     = "[" [ valeur { "," valeur } [ "," ] ] "]" ;
UNITE     = "mm" | "cm" | "m" | "km" | "ms" | "s" | "min" | "h" | "B" | "KB" | "MB" | "GB" ;
```

Un commentaire commence par `//`. `TEXTE_LONG` est entouré de `"""` et contient du Markdown. Les tailles sont décimales : 1 KB = 1 000 octets, 1 GB = 1 000 000 000 octets.

Règles lexicales précisées après la revue Codex du 2026-10-03 : un entier sans point ni unité est gardé exact (64 bits non signés) et ne passe jamais par un nombre flottant ; une unité se colle au nombre (`500KB`, jamais `500 KB`) ; un nom est fait de lettres, chiffres, `_` et points, et un nom de bloc commence par une majuscule. Le contrôle de cette suite (`verifier_suite.py`) vérifie que les cas sont bien formés ; il ne constitue pas le passage d'un moteur, qui demandera un exécuteur comparant arbre, diagnostics et journal.

**Vocabulaire en anglais** (`ADR-016`), avec une règle : un mot que les programmeurs connaissent déjà garde le sens qu'ils connaissent. Blocs utilisés : `Page`, `Text` (du texte sans rôle), `P` et `H1` à `H3` (un `Text` avec un rôle, `ADR-020`), `Button` (signal `tap`), `Image`, `Point` (capacités `enter` et `leave`), `World`, `On`. Paramètres : `name`, `title`, `children`, `inside`, `rules`, `effect`, `seed`, `brightness`, `fragments`, `budget`, `weight`, `source`, `text`. Dans une liste `children`, une phrase entre guillemets est un paragraphe à elle seule (`ADR-019`) : elle vaut un `P`. Une seule écriture par mot : un bloc commence par une majuscule, un paramètre par une minuscule.

**Styles** (`ADR-017`) : ils s'écrivent comme en CSS, après le bloc racine. Un style vise un type de bloc (`P { … }`) ou un nom à point (`.card { … }`), posé sur un bloc par `P.card(...)`. Le dernier `;` avant `}` est facultatif sur la même ligne ; partout ailleurs il est obligatoire. Réglages connus : `color`, `background`, `font-size`, `font-weight`, `font-style`, `font-family`, `text-align`, `border`, `border-radius`, `padding`, `margin`, `width`, `height`, `max-width`, `opacity`. Tailles en `px` ou en `%`.

## Ce que la suite contient, et ce qui manque

Six cas acceptés et seize cas refusés, centrés sur le premier sprint (le Big Bang), sur la règle des appels, sur le vocabulaire, sur le texte et sur les styles.

À ajouter ensuite, en reprenant les tests de la PR n° 2 : les archétypes composés, les relations spatiales, les lois, les durées (`for 3s`), les conflits entre phénomènes ; et depuis la PR n° 3 : l'entrée et la sortie sur plusieurs niveaux, avec une mémoire mesurée et non déclarée.

## Vérifier la suite elle-même

```bash
python verifier_suite.py
```

Ce script ne fait tourner aucun moteur. Il contrôle que chaque cas est complet et bien formé, pour qu'un cas bancal ne passe pas inaperçu.
