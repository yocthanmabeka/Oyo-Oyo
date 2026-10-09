# ADR-098 — Une abréviation expliquée, une date pour les machines, une adresse : `abbreviations:`, `<time>`, `Address`

- Statut : PROPOSITION (construit et essayé ; à valider par Yocthan)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #214 (« Dette du web : abbr, time et address »), ajoutée à la file à la demande de Yocthan le 2026-10-09 ; le grand tableau du web, où `abbr, time, address` était « non » (`docs/01-holocode/TABLEAU-WEB.md`) ; les dates (`ADR-067`).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- **Une abréviation** (« la MJC », « HTML ») gêne qui ne la connaît pas. HTML a `<abbr title="…">`, mais ce `title` ne se voit qu'au survol d'une souris : rien sur un téléphone, rien au clavier, et la plupart des lecteurs d'écran ne le lisent pas. Les guides d'écriture publics (GOV.UK, et le critère 3.1.4 des règles d'accessibilité) donnent la bonne pratique : écrire le sens en entier la première fois.
- **Une date** montrée « 14 novembre 2026 » se lit bien, mais un agenda ou un moteur de recherche ne la comprend pas. HTML a `<time datetime="2026-11-14">`, que presque personne n'écrit.
- **Les moyens de joindre** l'auteur d'une page (son adresse, son numéro) n'avaient pas de bloc. HTML a `<address>`.

## Décision

1. **`Page(abbreviations: [ Abbreviation("MJC", "Maison des jeunes et de la culture") ])`** : une abréviation et son sens, déclarés une fois pour toute la page.
   - Partout où elle vient comme un mot entier (« MJC », pas « MJC2 »), dans les textes de la page, le moteur la marque : `<abbr title="…">MJC</abbr>`. Pas dans du code (`` `MJC` ``).
   - **La première fois qu'elle vient dans un paragraphe** (`P`, `Text`), dans l'ordre de la page, le moteur écrit son sens juste après, entre parenthèses : « MJC (Maison des jeunes et de la culture) ». Le lecteur d'écran le lit, le téléphone le montre, l'impression aussi, avec ou sans JavaScript.
   - Pas dans un titre, un bouton, un lien : le sens s'écrit dans un paragraphe.
   - **Si la page écrit déjà ce sens quelque part** (en majuscules ou non), le moteur n'ajoute rien : l'auteur l'a expliqué.
   - Bornes : de 1 à 50 abréviations ; une forme courte de 1 à 20 signes (lettres, chiffres, et seulement `.`, `-`, `'`, `&` ou une espace entre eux) ; un sens d'une ligne, de 1 à 200 signes ; chaque forme courte une fois.
2. **Une date montrée devient un `<time>`**, sans rien écrire de plus : `{ouverture:date}` et `{ouverture:weekday}` (`ADR-067`) s'écrivent `<time datetime="2026-11-14">14 novembre 2026</time>`. Quand un geste change la date (`ouverture.add(7)`), la page change aussi `datetime`. Une date vide n'a pas de `datetime`.
3. **`Address(children: [ P("12 rue des Arts, Paris"), P("Téléphone : 01 23 45 67 89") ])`** : les moyens de joindre l'auteur de la page, en `<address>`. Ni titre (`H1` à `H6`) ni repère (`Header`, `Footer`, `Nav`, `Main`, `Aside`, une autre `Address`) dedans, même plus profond, comme en HTML. Au départ, le texte est droit : le navigateur le mettait en italique.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où écrire une abréviation | un bloc `Abbr` dans le texte (impossible : un texte n'a pas de blocs dedans) ; une marque dans chaque phrase ; **une liste déclarée une fois pour la page** | la page dit une fois ce que veut dire « MJC », et le moteur la reconnaît partout ; comme les abréviations de Markdown Extra et de Python-Markdown |
| Le mot | `Abbr`, le nom de HTML ; **`Abbreviation`** | un débutant ne devine pas `Abbr` ; comme `fonts: [ Font(…) ]`, `abbreviations: [ Abbreviation(…) ]` |
| Montrer le sens | le `title` seul, comme HTML ; un texte caché pour le seul lecteur d'écran ; **le sens écrit à la première venue, pour tous** | le `title` ne se voit pas au doigt et n'est pas lu ; un texte caché laisserait le téléphone sans le sens. Écrit une fois, il est lu, vu, imprimé, partout pareil (la parité du téléphone et de l'ordinateur) |
| Une date | un bloc `Time(…)` ; un format nouveau ; **rien de nouveau : chaque date montrée devient un `<time>`** | le moteur sait déjà ce qui est une date (`ADR-067`) : l'auteur n'a rien à écrire, et aucune date montrée n'est oubliée |
| Les moyens de joindre | `Contact` ; **`Address`, le nom de HTML** | sur le web, « Contact » veut dire un formulaire, et nos leçons nomment un formulaire `Contact` (`Form(name: Contact)`) ; « adresse » a les mêmes deux sens en français qu'en anglais (une rue, ou l'adresse d'une page) : `Page(address:)` (`ADR-091`) est un paramètre, en minuscules, `Address` un bloc |

## Ce qui est refusé, et pourquoi

- `Abbreviation` hors de `abbreviations:` : elle se déclare pour toute la page.
- Une abréviation déclarée deux fois, une forme courte trop longue ou avec des signes du texte enrichi (`*`, `` ` ``, `{`), un sens vide ou sur plusieurs lignes, plus de 50 abréviations.
- Un titre ou un repère dans `Address` : HTML ne le permet pas, et ce n'est pas un moyen de joindre.

## Les défauts du web évités

- **Le `title` invisible** au doigt et au clavier, et muet pour la plupart des lecteurs d'écran : ici, le sens est écrit, une fois.
- **Le même `<abbr title>` à répéter** à chaque venue, ou oublié : ici, déclaré une fois, marqué partout.
- **La date que seuls les humains lisent** : ici, toute date montrée l'est aussi pour les machines, et reste juste quand elle change.
- **L'italique imposé** à une adresse par le navigateur : ici, le texte reste droit.

## Dettes

- Une heure seule (« 14 h 30 ») n'est pas encore un `<time>` : la page n'a pas de valeur d'heure, seulement `{hour}` et `{minute}`.
- Une abréviation dans une liste qui change (`Repeat(over:)`) est marquée, mais son sens ne s'y écrit pas : la liste se redessine seule.
- Un lien pour écrire ou appeler (`mailto:`, `tel:`) dans une `Address` n'existe pas encore.

## Critères de validation

- Tests du moteur : `an_abbreviation_is_marked_everywhere_and_explained_once` (marquée dans un titre, un paragraphe et un bouton ; son sens écrit une fois, dans le premier paragraphe ; ni dans « HTML5 », ni dans « XHTML », ni dans du code) ; `a_page_that_writes_the_meaning_is_not_explained_twice` (et une page sans abréviation n'en garde aucune) ; `abbreviations_are_checked` (sept refus) ; `a_date_shown_is_a_time_for_machines` ; `an_address_holds_ways_to_reach_the_author` (et quatre refus) ; `dates_are_compared_shifted_and_shown`, mis à jour.
- Dans Chrome : « une abréviation expliquée une fois, une date lisible par les machines, une adresse (leçon 121) » (le sens écrit une fois, quatre abréviations marquées ; `datetime` suit la date après un toucher ; l'adresse dans le pied de page). Sans la mise à jour de `datetime` dans la page, l'essai rate.
- Leçon `121-une-abreviation-une-date-une-adresse.holo`.
