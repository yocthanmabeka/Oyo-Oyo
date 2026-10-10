# ADR-103 — Travailler un texte : des majuscules, sa longueur, le couper, le découper

- Statut : ACCEPTÉ (fait et validé : Yocthan, 2026-10-09, « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #232 (« Dernière dette du web : travailler un texte (majuscules, longueur, découper) »), l'une des douze dernières dettes du web, validées d'avance par Yocthan le 2026-10-09 ; le grand tableau du web, où « texte (majuscules, longueur, découper) » était « en partie » ; `ADR-043` (les formats après deux-points), `ADR-062` (les listes calculées), `ADR-063` (comparer des textes, à la lettre près), `ADR-041` (`text-transform`), `ADR-068` (la longueur d'un champ).
- Validation : Yocthan, le 2026-10-09, d'avance, avec les douze dernières dettes du web.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Quatre travaux reviennent sur le web :
  - un compteur sous un message, « 12 caractères sur 140 » ;
  - un code ou un nom montré en capitales ;
  - l'aperçu d'un texte long, coupé, dans une liste d'articles ;
  - des étiquettes écrites d'une traite, « art, peinture, Paris ».
- HoloCode savait montrer un texte (`{name}`), le comparer (`ADR-063`) et chercher dedans (`Filter(contains:)`), mais pas le travailler.
- Écrits à la main en JavaScript, ces travaux ont leurs pièges :
  - `.length` compte des unités de seize bits, pas des lettres : `"👍".length` vaut 2, une famille 👨‍👩‍👧 vaut 8, un drapeau 🇫🇷 vaut 4, et « é » écrit en deux morceaux (e + accent) vaut 2 ;
  - `slice` et `substring` coupent un émoji en deux (il reste un « � ») ou séparent un accent de sa lettre ;
  - `toUpperCase()` ignore la langue : en turc, « i » devient « I » au lieu de « İ » ;
  - `"a, b,".split(",")` garde les espaces et rend un morceau vide, `["a", " b", ""]`, et ne connaît qu'une virgule : ni « ， » ni « 、 » du chinois, ni « ، » de l'arabe.
- Le CSS a `text-transform`, `text-overflow: ellipsis` et `line-clamp` (`ADR-069`) : ils changent l'allure d'un bloc entier, à l'écran seulement. Ils ne s'écrivent pas dans le titre de l'onglet, ni dans un morceau de phrase, et ne coupent pas à un nombre de caractères.

## Décision

```holo
Page(
  title: "Message",
  state: State(code: "ab-12", message: "", keywords: "art, peinture, Paris"),
  computed: [ Split(name: tags, from: keywords, by: ",") ],
  children: [
    Input(value: code, label: "Your discount code"),
    P("Printed as: {code:upper}"),
    Input(value: message, label: "Your message", lines: 3, max: 140),
    P("{message:length} characters out of 140. Preview: {message:max40}"),
    Input(value: keywords, label: "Keywords, separated by commas"),
    P("{tags} tag(s)"),
    Repeat(over: tags, children: [ Text("#{item:lower}") ]),
  ],
)
```

1. **Quatre formats de plus, pour un texte** (`ADR-043`) : ils changent ce qu'on montre, jamais ce que le visiteur a écrit.
   - `{code:upper}` : en majuscules ; `{code:lower}` : en minuscules.
   - `{message:length}` : le nombre de caractères.
   - `{message:max40}` : au plus 40 caractères, « … » compris ; de `max2` à `max2000`.
2. **Un caractère est ce qu'une personne compte comme une lettre** : une grappe de graphèmes, selon les règles d'Unicode (UAX #29), celles de `Intl.Segmenter` dans Chrome.
   - 👍🏽, 🇫🇷, 👨‍👩‍👧, « é » écrit en deux morceaux, « क्षि » en devanagari : un caractère chacun.
   - Le moteur garde une table des lettres d'Unicode (16.0), fabriquée par `moteur/outils/graphemes.py` (711 plages, moins de 3 Ko).
3. **Les majuscules suivent la langue de la page** (`Page(lang:)`) :
   - « ß » devient « SS » ; en turc et en azéri, « i » devient « İ » et « I » devient « ı » ;
   - en grec, les accents tombent en majuscules (« Ελλάδα » → « ΕΛΛΑΔΑ ») et le sigma final s'écrit « ς » en minuscules.
4. **Couper** : un texte plus long que la place est coupé à la fin d'un mot, si l'on garde ainsi au moins la moitié de la place, sinon au milieu du mot (un mot très long, le chinois, le thaï) ; jamais au milieu d'une lettre. « … » dit qu'il continue, et la ponctuation qui pendrait avant lui (« , », « : », « - ») est retirée.
5. **Partout où un texte se montre** :
   - dans une phrase, et dans le titre de l'onglet (`Page(title: "Code {code:upper}")`) ;
   - dans les lignes d'une liste, pour un champ (`{item.title:max40}`) ou le texte de l'élément (`{item:upper}`) ;
   - sans JavaScript (`holo serve`) : le serveur fabrique la page avec le même moteur.
6. **Découper** : `computed: [ Split(name: tags, from: keywords, by: ",") ]`, une liste calculée (`ADR-062`) qui suit son texte.
   - Chaque morceau perd ses blancs autour ; les morceaux vides sont oubliés ; deux cents au plus, chacun dans la longueur d'un élément de liste.
   - `by: ","` coupe aux virgules de toutes les écritures (, ، 、 ， ﹐ ﹑ ､ ՝ ߸ ፣) ; `by: ";"`, aux points-virgules (; ؛ ； ፤ ⁏) ; `by: " "`, aux blancs (des mots) ; `by: lines`, à chaque ligne ; un autre texte, de un à dix signes, tel qu'il est écrit (`by: " - "`).
   - La liste se montre comme une autre : `Repeat(over: tags)`, `{tags}` pour leur nombre, `If(tags, over: 5)`, `suggestions: tags`, ou un `Filter(from: tags, …)` écrit après elle.
7. **La valeur montrée voyage dans l'état sous son propre nom** : `code:upper='AB-12`, `message:length=12`. La page l'écrit telle quelle, sans rien calculer ; l'arbitre ne la relit jamais. Comme les nombres de jours (`ADR-067`), elle se refait d'après le texte.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Les majuscules | le style `text-transform` (il existe : l'allure d'un bloc entier) ; une demande qui change la valeur, `code.upper()` ; **un format, `{code:upper}`** | le texte écrit reste ; le format se pose dans une phrase, dans le titre de l'onglet, et sans JavaScript. Pour l'allure d'un bloc entier, le style suffit, et reste |
| Le mot | `uppercase` et `lowercase` (CSS, Angular) ; `upcase` et `downcase` (Ruby, Liquid) ; **`upper` et `lower`** (Python, SQL, Django, Jinja) | les plus courts, et ceux que l'issue proposait ; `uppercase` est refusé avec le bon mot (`ADR-037`) |
| Compter | les unités de seize bits (`.length`, `maxlength`) ; les points de code (Rust, Python) ; **les lettres comme une personne les compte** (Swift, `Intl.Segmenter`, le compteur de Flutter) | « 12 caractères sur 140 » doit dire ce que le visiteur voit : un émoji est un caractère |
| La longueur | un compteur posé d'office sous chaque champ, comme Flutter (`maxLength`) ; un nombre calculé, `Length(name:, of:)` ; **un format, `{message:length}`** | le plus court pour « 12 caractères sur 140 », posé où l'auteur le veut ; un compteur d'office aurait changé toutes les pages qui ont un `max:` |
| Couper | `text-overflow` et `line-clamp` (à l'écran, selon la largeur) ; `{bio:40}` ; **`{bio:max40}`** | `max` dit déjà « au plus tant de caractères » pour un champ de texte (`Input(max: 140)`) ; `{bio:40}` se confondrait avec `{minute:00}` |
| Où couper | à la lettre près (Django, Liquid) ; au mot près seulement ; **à la fin d'un mot si l'on garde la moitié de la place, jamais dans une lettre** | un aperçu se lit mieux par mots entiers ; un mot très long ou une écriture sans espaces se coupe quand même |
| Découper | une demande qui remplit une liste d'un geste ; **une liste calculée, `Split`** | elle suit le texte pendant qu'on écrit, comme `Filter` ; rien à garder ni à relire |
| Le séparateur | la seule virgule ASCII ; **les virgules de toutes les écritures pour `","`** | le visiteur écrit avec le clavier de sa langue : « 北京，上海 » doit donner deux étiquettes |

## Ce qui est refusé, et pourquoi

- Un format de texte sur un nombre (`{cart:upper}`), sur une liste (`{tasks:length}` : son nombre s'écrit `{tasks}`) ou sur une date (ses formats sont `date` et `weekday`).
- `{code:uppercase}`, `{code:toUpperCase}`, `{message:len}` : refusés avec le bon mot. `{bio:max1}`, `{bio:max}`, `{bio:truncate40}` : `max` suivi d'un nombre de 2 à 2000. `{name:capitalize}` : une majuscule à chaque mot reste un style, `text-transform: capitalize`.
- `{item:upper}` hors d'une répétition ; `{item:00}` (le texte de l'élément prend les formats d'un texte).
- `Split` sans `from`, sans `by`, un séparateur vide ou sur plusieurs lignes, un `from` qui n'est pas un texte de la page, un nom déjà pris, un réglage inconnu.
- Changer une liste découpée (`tags.push(…)`) ou la réordonner (`reorder: true`) : elle suit son texte ; on change le texte.

## Les défauts du web évités

- **Un émoji qui compte pour deux, quatre ou huit** (`.length`) : ici, pour un, comme le voit le visiteur ; vérifié dans Chrome contre `Intl.Segmenter`.
- **Un émoji ou un accent coupé en deux** (`slice`) : ici, jamais.
- **Des majuscules qui ignorent la langue** (`toUpperCase()` en turc) : ici, la langue de la page.
- **Des morceaux vides et des blancs gardés** (`split`) : ici, oubliés ; et les virgules de toutes les écritures.
- **Une coupe qui n'existe qu'à l'écran** (`text-overflow`) : ici, le texte coupé est le même partout, dans le titre de l'onglet et sans JavaScript.

## Dettes

- La limite d'un champ (`max:`) compte encore comme avant (`ADR-027`, `ADR-068`) : en signes écrits, pas en lettres ; un émoji peut y compter pour plus d'un. À aligner sur le compte des caractères, en changeant aussi le `maxlength` que le navigateur applique.
- Pas de condition sur une longueur (`If` sur `{message:length}`) : `Input(min:)` vérifie la plus courte à l'envoi.
- Pas encore : rejoindre une liste en un texte ; une majuscule seulement au début ; les accents du grec ancien (polytonique) et la règle du point en lituanien.
- Une répétition écrite dans le fichier (`Repeat(items: …)`) ne prend pas `{item:upper}`.
- Quand Unicode changera, refaire la table : `python moteur/outils/graphemes.py`.
- Le grand tableau du web : « texte (majuscules, longueur, découper) » passe de « En partie » à « Oui », à la fusion.

## Critères de validation

- Tests du moteur :
  - `letters_are_counted_as_a_person_counts_them` : les émojis, leur couleur de peau, une famille, les drapeaux (l'Écosse aussi), le devanagari, le tamoul, le thaï, l'arabe, le coréen ;
  - `a_text_is_written_in_capitals_or_small_letters_in_its_language` : le français, le turc, le grec, l'allemand ;
  - `a_text_is_cut_at_the_end_of_a_word_never_inside_a_letter` ; `a_text_is_split_into_a_list` ;
  - `a_page_shows_a_worked_text_and_keeps_what_was_written` : la page, l'état, une saisie, le turc, les virgules du chinois, une ligne publiée, un état falsifié, le titre de l'onglet, le champ d'une fiche ;
  - `what_a_worked_text_refuses` : dix-sept refus.
- Dans Chrome : « travailler un texte : des majuscules dans la langue, les caractères comptés comme Intl.Segmenter, un aperçu coupé, découper ; sans JavaScript aussi (leçon 126, serve) ».
  - La page fabriquée par le serveur montre déjà « AB-12 », 71 caractères, l'aperçu et les étiquettes ;
  - « straße » devient « STRASSE » ; huit textes difficiles comptés comme `Intl.Segmenter` (et non comme `.length`) ;
  - cinquante « é » écrits en deux morceaux : un aperçu de quarante lettres, aucune coupée ; une ligne publiée, coupée à la fin d'un mot ;
  - « 北京，上海、 广州,, » donne trois étiquettes ; aucune valeur montrée dans une zone que le lecteur d'écran annoncerait à chaque lettre ;
  - sans JavaScript, avec `holo serve` : « istanbul ılık » devient « ISTANBUL ILIK », et les étiquettes suivent ;
  - l'essai rate si le moteur compte les points de code au lieu des lettres (vérifié).
- Leçon `126-travailler-un-texte.holo`.
