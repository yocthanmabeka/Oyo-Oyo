# Piste 6 — Interactions riches

> Statut : EXPLORATION. Avis de Claude, pas une décision.
>
> **Réparé le 2026-10-07, après cette exploration** : le pincement au doigt (PR 141) et Échap qui ne fermait plus une fenêtre quand la page écoute `Key.escape` (PR 145). Vérifié dans Chrome sans fenêtre, avant et après.

## Ce que Codex demandait

Sa ligne, dans l'issue #82 :

> | **6** | **Interactions riches** : changement de champ, soumission, focus, survol, défilement ; menus, accordéons et fenêtres de dialogue. | Construire une interface qui répond précisément au visiteur, au doigt comme au clavier. |

## État vérifié (main, 7a48def, 2026-10-07)

Remarque de relecture : `main` est passé à `1119361` (PR 139) ; ce changement ne touche aucune ligne citée ici (vérifié par `git diff --stat 7a48def 1119361` : `JOURNAL.md`, `bin/holo.rs`, `lib.rs`, et des tests à partir de `flat.rs:1566`).

**En bonne partie fait** : le survol, le clavier, le pli, la fenêtre, l'apparition en descendant existent et sont décidés. **Mais une régression récente casse le pincement au doigt**, et trois défauts d'accessibilité sont mesurés dans Chrome (une fenêtre sans nom ; le focus perdu après un passage ; le focus perdu quand le bouton touché disparaît, sur le panier du site de référence), plus un quatrième vu à la relecture (Échap ne ferme plus une fenêtre quand la page écoute `Key.escape`). Ce sont eux qu'il faut traiter d'abord ; le reste est du confort.

### La régression : le pincement ne marche plus (mesuré sur PC)

Le passage du code en anglais (`ADR-060`, commit `172931b`) a remplacé partout le mot `touches` (la fonction Rust des touches du clavier, devenue `keypresses`). Il a aussi remplacé `event.touches`, la liste des doigts posés sur l'écran, qui fait partie du navigateur. `event.keypresses` n'existe pas : chaque toucher lève une erreur.

- `moteur/web/page.html:219` : `if (event.keypresses.length > 1) load();`
- `moteur/web/page-engine.js:879`, `880`, `883`, `885`, `891` : le même mot, dans le pincement de la page.
- Avant la traduction, `git show 0fde532^1:moteur/web/page.html` ligne 196 : `if (evenement.touches.length > 1) charger();`.

Mesure, Chrome 154 sans fenêtre sur le PC (SwiftShader), deux doigts simulés par le protocole de Chrome, script `essais-2-6-7/pincer.sh` :

Sortie abrégée (la sortie complète est dans `essais-2-6-7/pincer-sortie.txt`) :

```text
=== A. Leçon 1 (page légère, sans moteur) : pincer, puis Ctrl + molette ===
après le pincement → {"erreurs":["Uncaught TypeError: Cannot read properties of undefined (reading 'length')","Uncaught TypeError: Cannot read properties of undefined (reading 'length')"],"moteur":false,"zoom":"(aucun)"}
après Ctrl + molette → {"erreurs":[…les deux mêmes…],"moteur":true,"zoom":"(aucun)"}
=== B. Leçon 9 (moteur chargé par ?values) : pincer, puis Ctrl + molette ===
après le pincement → {"erreurs":[…quatre fois la même erreur…],"moteur":true,"zoom":"(aucun)"}
après Ctrl + molette → {"erreurs":[…],"moteur":true,"zoom":"scale(2)"}
```

À la souris, le zoom marche (`scale(2)`) ; au doigt, rien, et des erreurs. Or `page.html:22` coupe le pincement du navigateur (`touch-action: pan-x pan-y`) pour le confier au moteur : **sur un téléphone, on ne peut donc plus zoomer du tout**, ni pour lire plus gros, ni pour entrer dans les points. Non vérifié sur un téléphone : pas d'appareil ici.

**Même cause, hors de cette piste :** le module enfermé (`ADR-045`) ne marche plus. Le code de la boîte, écrit dans un texte, a gardé ses mots français (`octets`, `entree`, `debut`, `sortie`, `raison` : `page-engine.js:1173-1184`) ; le code qui lui parle a été traduit (`bytes`, `entry`, `start`, `output`, `reason` : `page-engine.js:1188-1218`). Avant la traduction, les deux côtés disaient `octets`, `entree`, `debut`, `sortie` (`git show 0fde532^1:moteur/web/page-moteur.js`, lignes 1129-1174). Mesure, script `essais-2-6-7/module.sh`, leçon 69, bouton « Calculer la somme » :

```text
→ {"modules":[{"name":"Somme","ok":false,"raison":"WebAssembly.instantiate(): Argument 0 must be a buffer source or a WebAssembly.Module object"}], …}
```

**Pourquoi personne ne l'a vu :** les tests automatiques (`.github/workflows/tests.yml`) lancent le Rust et les prototypes Python ; aucun ne lance une page dans un navigateur. Les essais dans Chrome de `ADR-060` (`docs/02-gouvernance/adr/ADR-060-code-en-anglais.md:31`) citent la leçon 9, sans dire qu'elle a été essayée au doigt, et ne citent pas la leçon 69.

### Ce qui existe

| Demande de Codex | Où c'est | Décision, leçon |
|---|---|---|
| Survol | `On(Card.hover)`, `On(Card.hoverEnd)` sur tout bloc nommé qui se voit (`moteur/src/rules.rs:12-22`) ; à la souris, au clavier, au doigt (`page-engine.js:1335-1374`) ; rejoué si le moteur arrive pendant le survol (`page.html:220-235`) ; un survol n'emmène jamais ailleurs (`rules.rs:247-249`, sonde P6-14) ; l'allure au survol, au focus, à l'appui (`flat.rs:385-395`) | `ADR-036`, `ADR-039`, leçons 37, 45 |
| Focus | l'allure `focus:` (`:focus-visible`, `flat.rs:389`) ; un bloc écouté au survol reçoit le focus au clavier (`flat.rs:207-217`) ; le focus déclenche `hover` (`page-engine.js:1357-1358`) | `ADR-036`, `ADR-039` |
| Changement de champ | le champ est lié à sa valeur, sans règle (`page-engine.js:1427-1433`) ; `When` réagit quand un **nombre** change (case, glissière, champ de nombre) | `ADR-027`, `ADR-028` |
| Soumission | `Contact.send` par un bouton ; voir la piste 2 | `ADR-042` |
| Défilement | `Enter(…, inView: true)` : un bloc entre quand il arrive à l'écran, sans écouter le défilement (`page.html:146-164`) ; un lien vers un endroit de la page (`A(to: "#Hours")`) | `ADR-061`, `ADR-042`, leçons 56, 78 |
| Clavier | `Key.enter`, `Key.escape`, les lettres, les chiffres ; jamais Tab ; les touches à une lettre se coupent | `ADR-061`, leçon 77 |
| Accordéons | `Details(summary:, children:, open:)` → `<details>`, marche sans le moteur (`flat.rs:1063-1075`) | `ADR-042`, leçon 62 |
| Fenêtres | `Dialog(name:)`, `open`, `close` (`rules.rs:38`) → `showModal()` (`page-engine.js:1238-1242`) : le clavier reste dans la fenêtre, Échap la ferme | `ADR-042`, leçon 63 |
| Au doigt | `tap`, `drag` sur un plateau, toucher survole | `ADR-026`, `ADR-028`, `ADR-039` |

### Ce qui manque (sondes `essais-2-6-7/sondes.sh` et `sondes2.sh`, mesures dans Chrome)

1. **La fenêtre n'a pas de nom.** Mesure, script `essais-2-6-7/focus-et-noms.mjs`, leçon 63 : l'arbre d'accessibilité de Chrome montre `{"role":"dialog","nom":""}`. Le HTML n'a ni `aria-labelledby` ni `aria-label` (`flat.rs:1082`). Un lecteur d'écran dit « boîte de dialogue », sans dire laquelle. Le bouton de fermeture s'appelle « Fermer » même dans une page `lang: "en"` (sonde P7-19, `flat.rs:1082`). Ce qui marche (mesuré) : le focus entre dans la fenêtre (sur ✕) ; Échap la ferme et rend le focus au bouton « Vider le panier ». **Sauf si la page écoute `Key.escape`** (défaut vu à la relecture) : le moteur appelle alors `preventDefault()` sur Échap dès que le focus n'est pas dans un champ (dans la fenêtre, il est sur ✕) (`page-engine.js:1414-1422`), et Chrome ne ferme plus la fenêtre. Mesuré, Chrome 154 sans fenêtre, moteur réel, page d'essai `relecture-2-6-7/fauxdepot/exemples/echap.holo` (une `Dialog` et `On(Key.escape, effect: n.add(1))`), servie par un second serveur local (`HOLO_REPO` pointé sur ce dossier, port 8099, arrêté ensuite) : après Échap, `{"ouverte":true,"compte":"1"}` ; la même page sans la règle : `{"ouverte":false}`. La croix ferme encore ; `holo check` accepte ce mélange (P6-18).
2. **On ne sait pas quand la fenêtre se ferme.** Aucun signal (`On(Thanks.closed)` refusé, P6-06), aucune écoute de l'événement `close` dans `moteur/web/` (recherche : aucun `addEventListener("close"`). Fermée par la croix ou Échap, la fenêtre ne prévient pas les règles.
3. **Le focus se perd après un passage.** Mesure, leçon 7, Entrée sur « Entrer dans l'atelier » : titre « Atelier », adresse `#Atelier`, **focus sur `body`**, rien d'annoncé. La page est remplacée (`page-engine.js:549-566`) sans poser le focus ; seul le carrefour le fait (`page-engine.js:1000`). Au clavier ou au lecteur d'écran, on repart du haut de la page sans savoir qu'on a changé de lieu.
   **Le focus se perd aussi quand son bloc disparaît.** Mesure sur le site de référence (`essais-2-6-7/focus-vider.mjs`) : dans `panier.holo`, « Vider le panier » est rangé dans `If(count, over: 0)` (`exemples/site-reference/panier.holo:54-57`) ; au clavier, « + » puis « Vider le panier » donnent : `bouton visible ? → false ; focus → body`.
4. **Un menu.** Aucun bloc. Deux façons acceptées aujourd'hui : `Details` dans `Nav` (sonde P6-16, marche sans le moteur, mais ne se ferme ni à Échap ni en touchant ailleurs) ; deux boutons dans un `If`/`else` (P6-09), qui cache le bouton qui avait le focus, sans `aria-expanded`. Pas de demande « basculer » (`menu.toggle` : P6-08 propose `menu.toggle(1)`, que P6-15 refuse ensuite : un message qui égare, `rules.rs:282-283`).
5. **Un accordéon où un seul pli s'ouvre.** `Details(group:)` refusé (P6-10) ; une règle ne peut ni ouvrir ni fermer un pli (`Faq.open` : « un « Details » offre rien », P6-05). Le message lui-même est mal écrit : « offre rien » au lieu de « n'offre rien », et de même « émet rien » (P6-11) ; la cause est `list_all`, `rules.rs:316-322`.
6. **Réagir à un choix.** `When(size, is: "M")` est refusé (P6-02) : `When` ne guette que les nombres (`moteur/src/state.rs:78-79`). Le message dit « If(size, is: …) » alors qu'on a écrit `When` (`state.rs:85`). Pas de signal `change` (P6-01).
7. **Donner le focus, ou le suivre.** Ni signal `focus` (P6-03), ni capacité (P6-12) ; le focus ne déclenche que `hover`.
8. **Défilement : la position.** Rien ne dit où l'on est dans la page (`docs/01-holocode/COMPARAISON-WEB.md:152`, toujours vrai). Un en-tête qui reste en haut (`position: sticky`) est impossible : `position` est refusé dans un style (`ADR-017`).
9. **Un bouton désactivé** : `Button(disabled:)` refusé (P6-13). C'est voulu jusqu'ici : « pour elle, on cache le bouton » (`GUIDE.md:740`, `ADR-028:84`) ; mais c'est ce qui fait perdre le focus (points 3 et 4).

### Documents en retard

- `docs/01-holocode/TABLEAU-WEB.md:612` : « pincer, zoomer — 100 % » (cassé depuis `ADR-060`) ; `:537` `Dialog` à 100 % (sans nom ni signal de fermeture) ; `:536` `Details` à 100 % (ni groupe ni règle).
- `COMPARAISON-WEB.md:105-106` : `details` et `dialog` « manque » (faits) ; `:165` « Animations écrites par l'auteur : manque » (`Enter`, `Loop`, `Scenes` existent).
- `docs/01-holocode/NOMS.md:230` (`details, summary, dialog`) et `:237` (« Les événements de survol, de clavier, de défilement ») dans « Pas encore là » : faits, sauf le défilement (seulement l'apparition, `Enter(…, inView: true)` ; aucun signal, P6-11).
- `docs/01-holocode/GUIDE.md:1519` (« Ctrl + molette, ou pincer ») : vrai dans l'intention, faux aujourd'hui au doigt.
- Vu en passant, hors de cette piste : `exemples/motion/README.md:30-32` décrit un défaut corrigé depuis (`ADR-033`, correction du 2026-10-06).

## Le scénario du site de référence

**Les cas de Codex :** F15, « Parcours lecture, panier, atelier et retour au clavier puis au lecteur d'écran » : « Contrôles nommés, focus visible, pas de piège ; retour à plat atteignable » (`proposals/GPT5.6/site-reference-2026-10-06/RECETTE.md:29`) ; et E02, « … puis ouverture du menu » (`RECETTE.md:36`). F15 est `NON EXÉCUTÉ` au premier passage (`exemples/site-reference/RECETTE-2026-10-06.md:20`).

**La tâche concrète sur le site construit :**

1. Sur un téléphone, à 360 pixels, le menu commun a sept liens (`exemples/site-reference/commun.holo:12-20`). Un bouton « Menu » les ouvre ; Échap, un toucher ailleurs ou le choix d'un lien le referment ; le focus revient au bouton.
2. Dans `panier.holo`, « Explorer l'atelier » mène au monde (`panier.holo:69`, `:137`) : au clavier, le focus doit arriver sur « L'atelier », et « Revenir au panier » doit le rendre au bouton d'où l'on est parti.
3. Toujours dans `panier.holo` (`points: Points(after: 3)`, `:17`) : pincer sur un téléphone doit grossir la page, puis la changer en points.

**Ce qu'il faut :** corriger le pincement (3) ; poser le focus après un passage (2) ; un menu qui se ferme comme on l'attend (1). Le nom de la fenêtre et son signal de fermeture servent aux autres pages.

## Options comparées

### A. Le pincement

| Option | Ce qu'on fait | Avantages | Défauts |
|---|---|---|---|
| **A1. Corriger, et un essai dans un navigateur** | `event.touches` aux six endroits ; un essai qui lance Chrome sans fenêtre (comme `moteur/outils/capture.mjs`, sans rien installer ; Chrome est fourni sur les machines Ubuntu de GitHub selon leur documentation, non vérifié ici ; `capture.mjs:20` cherche Chrome à son chemin Windows, réglable par `CHROME`, et `tests.yml` n'installe pas Node : il faudra `actions/setup-node` pour avoir Node 22, non vérifié) et échoue si un toucher lève une erreur | la régression ne revient pas ; le même essai attrape le module cassé | une séance de plus pour l'essai (estimation) |
| A2. Corriger seulement | six mots | immédiat | la prochaine traduction peut recasser |

### B. Un menu

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| B1. Le pli, en mieux | `Nav(children: [ Details(summary: "Menu", children: [ … ]) ])` (accepté) ; le moteur ferme le pli à Échap et quand on touche ailleurs, et rend le focus au résumé | `<details>` + quelques lignes de JS | `ExpansionTile` | aucun mot nouveau ; marche sans le moteur (sans Échap ni toucher ailleurs) | un pli pousse le contenu au lieu de passer par-dessus ; il faut le moteur pour Échap |
| **B2. Une bulle par-dessus** | `Button(name: MenuButton, text: "Menu", toggles: SiteMenu)`, `Popover(name: SiteMenu, children: [ … ])` | l'attribut `popover` et `popovertarget` (l'API Popover : par-dessus la page, se ferme à Échap et en touchant ailleurs, sans JS) | `MenuAnchor`, `PopupMenuButton`, `OverlayPortal` | marche **sans le moteur** (aucun Ko de plus), à une condition : aujourd'hui, tout toucher sur un bloc nommé fait venir le moteur (`page.html:200-214`), donc `MenuButton` le ferait venir, sauf à changer la page légère ; sert aussi aux bulles d'aide ; le navigateur dit lui-même si le menu est ouvert (l'état « développé » du bouton ; non vérifié ici au lecteur d'écran) | deux mots ; un troisième « ouvrant » après `Details` et `Dialog` ; la place exacte sous le bouton demande le positionnement par ancre du CSS, pas encore partout (non vérifié ici) |
| B3. Basculer une valeur | `On(MenuButton.tap, effect: menu.toggle())`, `If(menu, is: 1, children: [ Nav(…) ])`, `Button(…, expanded: menu)` | `aria-expanded` et du JS | `setState(() => open = !open)` | général | trois mots ; demande le moteur ; l'auteur doit relier le bouton et ce qu'il ouvre |
| B4. Rien (aujourd'hui) | deux boutons dans `If`/`else` (P6-09) | — | — | existe | le focus se perd ; pas d'`aria-expanded` |

### C. La fenêtre

| Option | Écriture HoloCode | HTML/CSS/JS | Avantages | Défauts |
|---|---|---|---|---|
| **C1. Le nom pris au premier titre, la croix dans la langue de la page, un signal** | rien pour le nom (`H2("Les nouveautés")` devient le nom) ; `On(News.closed, …)` | `aria-labelledby`, `dialog.addEventListener("close", …)` | le défaut disparaît pour tous les fichiers déjà écrits ; un seul mot nouveau | une fenêtre sans titre reste sans nom : le moteur doit alors la refuser, ou demander `label:` |
| C2. Un nom écrit | `Dialog(name: News, label: "Les nouveautés", …)` | `aria-label` | explicite ; `label:` existe déjà ailleurs | un oubli possible, comme `alt` avant `ADR-038` ; à rendre obligatoire faute de titre |

### D. Réagir à un choix (changement de champ)

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **D1. `When` et `If` comparent un texte** | `When(size, is: "Grand", effect: note.set(1))` | `radio.addEventListener("change", …)` | `onChanged` | aucun mot nouveau ; partagé avec la piste 2 ; garde « trois sortes de règles, pas plus » | une faute de frappe dans le texte : le moteur doit refuser un texte qui n'est pas une option du `Choice` |
| D2. Un signal `change` | `On(Size.change, effect: …)` | l'événement `change` | `onChanged` | familier | un mot ; sur le web `change` part quand on quitte le champ, en Flutter `onChanged` part à chaque lettre : deux sens |

### E. Le focus

| Option | Écriture HoloCode | HTML/CSS/JS | Avantages | Défauts |
|---|---|---|---|---|
| **E1. Le moteur tient le focus** | rien : après un passage, le focus va au premier titre du lieu (`tabindex="-1"`) et le titre est annoncé ; si le bloc qui a le focus disparaît (un `If`), le focus va à son remplaçant dans le même `If`/`else`, sinon au titre le plus proche avant lui | `element.focus()` écrit à la main à chaque mise à jour | aucun mot ; le défaut disparaît partout | une règle à bien borner : ne jamais déplacer le focus quand il n'était pas perdu |
| E2. Une capacité | `On(Go.tap, effect: NameField.focus)` | `input.focus()` | l'auteur décide | un mot ; facile à oublier, ou à mal employer (voler le focus) |

### F. Un accordéon (un seul pli ouvert)

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **F1. Un groupe** | `Details(summary: "…", group: faq, children: [ … ])` | `<details name="faq">` (le navigateur ferme les autres) | `ExpansionPanelList.radio` | sans le moteur ; un mot | les navigateurs anciens ouvrent tout (le contenu reste lisible) |
| F2. Un bloc qui les range | `Accordion(single: true, children: [ Details(…), … ])` | — | `ExpansionPanelList` | se lit d'un coup d'œil | un bloc de plus ; `Accordion` n'est un mot ni du HTML ni de Flutter |

### G. Le défilement

| Option | Écriture HoloCode | Avantages | Défauts |
|---|---|---|---|
| **G1. Rien de nouveau maintenant** | `Enter(…, inView: true)` (par exemple `enter: Enter(y: 40px, inView: true)` ; seul, `Enter(inView: true)` est refusé : « « Enter » dit ce qui bouge ») et `A(to: "#Top")` vers un bloc nommé `Top` existent | aucun coût | pas de « vous êtes ici » dans le menu, pas d'en-tête qui reste en haut |
| G2. Un signal ou une valeur | `On(Section.seen, …)`, `{scroll}` | permet le menu qui suit la lecture | `ADR-061` l'a écarté pour le mouvement ; écouter le défilement coûte de la batterie |
| G3. Un en-tête collé | `Header(stick: true)` | un besoin courant | touche à la disposition (`ADR-017`) : à voir avec la piste 4 |

### Noms (ADR-016)

| Nom proposé | Sens sur le web | Sens dans Flutter | Risque de confusion |
|---|---|---|---|
| `Popover` | l'attribut `popover` : une couche par-dessus la page, non modale, qui se ferme seule : le même sens | pas de widget `Popover` dans Flutter ; `MenuAnchor`, `OverlayPortal` | faible ; à distinguer de `Dialog` (modale). Pas de conflit avec les composants des exemples (`Menu`, `Panel`, `Pied`, `Entete`, `ArticleCard`) |
| `toggles:` (sur `Button`) | `popovertarget` (action par défaut : `toggle`), `togglePopover()` | aucun | faible ; préférable à `opens:`, qui se confondrait avec la capacité `open` et ne dit pas que le bouton referme |
| `group:` (sur `Details`) | `<details name>` ; le `name` d'un groupe de boutons ronds | `ExpansionPanelList.radio` | faible pour un paramètre ; `name` est déjà pris (le nom d'un bloc) ; noter que « group » sera tentant pour la 3D (un groupe d'objets) |
| `closed` (signal) | l'événement `close` de `dialog` | `showDialog` rend une promesse résolue à la fermeture | faible ; au passé, comme `sent` et `done` |
| `announce:` | `aria-live` | `SemanticsService.announce` | faible (partagé avec la piste 2) |
| **À éviter : `Menu`** | `<menu>` (une liste de commandes), le rôle ARIA `menu` (un menu d'application, avec les flèches : un piège connu pour la navigation d'un site) | `MenuAnchor`, `DropdownMenu` | **fort**, et un conflit réel : `exemples/site/commun.holo:5` et `exemples/site-reference/commun.holo:6` nomment un composant `Menu` ; un bloc `Menu` les ferait refuser (`moteur/src/components.rs:129-131`). Même risque pour `Panel` (`GUIDE.md:933`) |
| À éviter : `Accordion` | aucun élément ; le « motif accordéon » des guides du W3C | aucun widget de ce nom | moyen : un mot de plus, sans sens établi |

## Recommandation

1. **Tout de suite, sans décision de langage : corriger le pincement (A1)**, et ajouter l'essai dans un navigateur. Le même essai vérifiera la leçon 69 (module), cassée par la même traduction.
2. **C1 : la fenêtre nommée par son premier titre**, la croix dans la langue de la page, et le signal `closed`. Refuser une fenêtre sans titre ni `label:`.
3. **E1 : le moteur tient le focus** après un passage et quand le bloc focalisé disparaît.
4. **D1 : `When` et `If` comparent un texte** (avec la piste 2).
5. **B1 maintenant** (le pli dans `Nav`, que le moteur ferme à Échap et en touchant ailleurs), puis **B2 `Popover` et `toggles:`** si Yocthan veut un menu par-dessus la page sans moteur.
6. **F1 : `Details(group:)`.**
7. Pas maintenant : G2, G3 (avec la piste 4), une capacité `focus`, `Button(disabled:)`.

## Exemple d'auteur

Les lignes marquées `// proposé` sont une **écriture proposée** : elle n'existe pas. Sans rien écrire, et proposé aussi : le pincement réparé, le focus tenu par le moteur, le nom de la fenêtre pris à son titre.

```holo
Page(
  title: "Questions — L'atelier des mondes",
  lang: "fr",
  state: State(size: "", note: 0, asked: 0),
  keep: [asked],
  children: [
    Header(children: [
      Row(gap: 12px, align: between, children: [
        Text("L'atelier des mondes"),
        Button(name: MenuButton, text: "Menu", toggles: SiteMenu),                  // proposé : toggles
      ]),
      Popover(name: SiteMenu, children: [                                          // proposé : Popover
        Nav(label: "Menu principal", children: [                                   // proposé : label (piste 7)
          List(children: [ A("Accueil", to: "accueil.holo"), A("Catalogue", to: "catalogue.holo"), A("Panier", to: "panier.holo") ]),
        ]),
      ]),
    ]),
    H1("Questions"),
    Details(summary: "Livrez-vous à l'étranger ?", group: faq, children: [ P("Oui, partout en Europe, en cinq jours.") ]),   // proposé : group
    Details(summary: "Les tableaux sont-ils encadrés ?", group: faq, children: [ P("Oui, d'un cadre en chêne.") ]),   // proposé : group
    Choice(value: size, label: "Format", options: ["Petit", "Grand"]),
    If(note, is: 1, announce: true, children: [ P("Le grand format part en caisse de bois.") ]),   // proposé : announce
    If(asked, is: 0, children: [ Button(name: Ask, text: "Recevoir les nouveautés") ]),
    Dialog(name: News, children: [
      H2("Les nouveautés"),                                                         // proposé : devient le nom de la fenêtre
      P("Un courriel par mois, pas plus. Écrivez-nous depuis la page Contact."),
      Button(name: Ok, text: "D'accord"),
    ]),
  ],
  rules: [
    When(size, is: "Grand", effect: note.set(1)),                                  // proposé : When sur un texte
    When(size, is: "Petit", effect: note.set(0)),                                  // proposé : When sur un texte
    On(Ask.tap, effect: News.open),
    On(Ok.tap, effect: News.close),
    On(News.closed, effect: asked.set(1)),                                         // proposé : closed
  ],
)
```

Vérifié : refusé au premier mot proposé ; accepté une fois les mots proposés retirés (`Popover` remplacé par `Column`, les `When` sur un texte et `closed` enlevés) :

```text
$ holo.exe check essais-2-6-7/piste-06-exemple.holo
essais-2-6-7/piste-06-exemple.holo : ligne 10, colonne 48 : « Button » n'a pas de paramètre « toggles » ; paramètres possibles : name, text
$ holo.exe check essais-2-6-7/piste-06-sans-propose.holo
ok
```

**Le même en HTML, CSS et JavaScript**, qui fait exactement la même chose (menu par-dessus la page, un seul pli ouvert, note annoncée au choix « Grand », fenêtre nommée, fermeture guettée, valeur gardée, focus rendu au titre quand le bouton disparaît). Fichier : `essais-2-6-7/piste-06-jumeau.html`.

```html
<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Questions — L'atelier des mondes</title>
<style>
  .row { display: flex; flex-wrap: wrap; gap: 12px; align-items: center; justify-content: space-between; }
  .for-reader { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  fieldset { border: 0; padding: 0; }
</style>
</head>
<body>
<header>
  <div class="row">
    <span>L'atelier des mondes</span>
    <button type="button" popovertarget="site-menu">Menu</button>
  </div>
  <div id="site-menu" popover>
    <nav aria-label="Menu principal">
      <ul>
        <li><a href="accueil.holo">Accueil</a></li>
        <li><a href="catalogue.holo">Catalogue</a></li>
        <li><a href="panier.holo">Panier</a></li>
      </ul>
    </nav>
  </div>
</header>
<main>
  <h1 tabindex="-1">Questions</h1>
  <details name="faq"><summary>Livrez-vous à l'étranger ?</summary><p>Oui, partout en Europe, en cinq jours.</p></details>
  <details name="faq"><summary>Les tableaux sont-ils encadrés ?</summary><p>Oui, d'un cadre en chêne.</p></details>
  <fieldset>
    <legend>Format</legend>
    <label><input type="radio" name="size" value="Petit"> Petit</label>
    <label><input type="radio" name="size" value="Grand"> Grand</label>
  </fieldset>
  <p id="note" hidden>Le grand format part en caisse de bois.</p>
  <button type="button" id="ask">Recevoir les nouveautés</button>
  <dialog id="news" aria-labelledby="news-title">
    <form method="dialog"><button aria-label="Fermer">✕</button></form>
    <h2 id="news-title">Les nouveautés</h2>
    <p>Un courriel par mois, pas plus. Écrivez-nous depuis la page Contact.</p>
    <button type="button" id="ok">D'accord</button>
  </dialog>
  <div id="live" class="for-reader" role="status" aria-live="polite"></div>
</main>
<script>
  const $ = (id) => document.getElementById(id);
  const say = (text) => { $("live").textContent = ""; setTimeout(() => ($("live").textContent = text), 50); };
  let asked = 0;
  try { asked = Number(localStorage.getItem("questions:asked") ?? 0); } catch {}
  $("ask").hidden = asked === 1;
  for (const radio of document.querySelectorAll("input[name=size]")) {
    radio.addEventListener("change", () => {
      if (radio.value === "Grand" && $("note").hidden) { $("note").hidden = false; say($("note").textContent); }
      if (radio.value === "Petit") $("note").hidden = true;
    });
  }
  $("ask").addEventListener("click", () => $("news").showModal());
  $("ok").addEventListener("click", () => $("news").close());
  $("news").addEventListener("close", () => {
    asked = 1;
    try { localStorage.setItem("questions:asked", "1"); } catch {}
    $("ask").hidden = true;
    document.querySelector("h1").focus(); // le bouton qui avait le focus a disparu
  });
</script>
</body>
</html>
```

Mesuré (`node mesure-jumeaux.mjs`) : **3 185 octets bruts, 1 194 octets compressés, 19 lignes de JavaScript** ; syntaxe vérifiée par `node --check`. Ces chiffres sont ceux du fichier `essais-2-6-7/piste-06-jumeau.html`, avec ses commentaires ; le bloc montré ci-dessus (même code) pèse 2 923 octets bruts, 1 062 compressés, 19 lignes (relecture : `node relecture-2-6-7/mesure.mjs`). Essayé dans Chrome sans fenêtre (`node jumeaux-essai.mjs`) :

```text
6 · menu ouvert après le bouton ? → true
6 · menu ouvert après Échap ? → false
6 · plis ouverts après avoir ouvert le second → false,true
6 · note visible après « Grand » → true ; annonce → "Le grand format part en caisse de bois."
6 · après Échap dans la fenêtre : bouton caché ? → true ; focus → H1
```

Dans ce jumeau, le menu et les plis marchent sans JavaScript ; c'est aussi ce que vise l'option B2 pour HoloCode.

## Par couche

- **Langage** : `Popover`, `toggles:`, `group:`, le signal `closed`, `announce:` (avec la piste 2), `If` et `When` sur un texte. Une règle de plus : une `Dialog` a un titre ou un `label:`.
- **Moteur (Rust)** : `blocks.rs` (nouveaux réglages) ; `flat.rs:1063-1085` (`name` du pli, `aria-labelledby` de la fenêtre, croix traduite selon `lang`) ; `rules.rs:10-43` (`closed` pour `Dialog`) ; `state.rs:60-110` (comparer un texte) ; `rules.rs:282-283` (ne plus proposer `menu.toggle(1)`) ; `state.rs:85` (dire `When` quand c'est un `When`) ; `rules.rs:316-322` (« n'offre rien », « n'émet rien »).
- **Enveloppe navigateur** : `page.html:219` et `page-engine.js:879-891` (`event.touches`) ; `page-engine.js:1173-1218` (les mots de la boîte du module, hors piste) ; écouter `close` sur chaque fenêtre et émettre `closed` ; laisser Échap fermer une fenêtre ouverte même quand la page écoute `Key.escape` (`page-engine.js:1414-1422`) ; poser le focus après `displaySite` (`page-engine.js:549-566`) et quand le bloc focalisé disparaît (`showValues`, `page-engine.js:504-514`) ; fermer un pli de `Nav` à Échap et au toucher ailleurs (option B1).
- **Services serveur** : rien.
- **Outils** : un essai dans Chrome sans fenêtre, lancé par `tests.yml`, sur quelques leçons (9 : pincer ; 63 : la fenêtre ; 69 : le module ; 7 : le focus après un passage).

## Dépendances

- A1 ne dépend de rien. Son essai dans un navigateur rejoint la question en attente : « installer Playwright et axe-core ? » (message précédent à Yocthan, point 5). L'essai proposé ici n'installe rien : il parle à Chrome comme `moteur/outils/capture.mjs` (Node 22 a déjà `WebSocket` ; ici, `node --version` rend `v22.21.0` ; sur les machines de GitHub, la version de Node n'est pas vérifiée).
- D1 et `announce:` sont partagés avec la piste 2 : une seule construction.
- `Nav(label:)` vient de la piste 7.
- G3 (en-tête collé) dépend de la piste 4 (disposition).
- B2 et la piste 9 (graphismes) : une bulle d'aide attachée à un bloc (`ADR-036` parlait d'un futur `Anchor`) pourrait être un `Popover` ; à trancher ensemble.

## Coût

| Travail | Moteur | Poids transféré | Travail |
|---|---|---|---|
| A1 pincement + essai navigateur | 6 mots ; ~80 lignes pour l'essai (estimation) | 0 | 0,5 séance, plus 1 pour l'essai dans `tests.yml` (estimation) |
| Module (hors piste) | ~6 mots dans `page-engine.js:1173-1184` | 0 | inclus dans l'essai |
| C1 fenêtre | ~30 lignes de Rust, ~15 de JS (estimation) | +0,2 Ko compressé (estimation) | 0,5 séance (estimation) |
| E1 focus | ~60 lignes de JS (estimation) | +0,5 Ko (estimation) | 1 séance, essais au clavier compris (estimation) |
| B1 pli de menu | ~25 lignes de JS (estimation) | +0,2 Ko (estimation) | 0,5 séance (estimation) |
| B2 `Popover` + `toggles:` | ~120 lignes de Rust, ~10 de JS (estimation) | **0 Ko de moteur à l'ouverture** ; au premier toucher aussi, si la page légère cesse de faire venir le moteur pour un bouton que seule une bulle écoute (`page.html:200-214` le fait venir aujourd'hui) (estimation fondée sur le comportement de `popover`) | 1,5 séance, leçon comprise (estimation) |
| F1 groupe | ~20 lignes de Rust (estimation) | 0 | 0,5 séance (estimation) |

**Poids mesuré aujourd'hui** (script `essais-2-6-7/poids.sh`, Chrome sans fenêtre, serveur local en Brotli, ouverture sans geste, PC) :

```text
62-plis.holo    : {"total_ko":8.7,"moteur":false}
63-fenetre.holo : {"total_ko":8.8,"moteur":false}
```

Un pli reste léger (8,7 Ko) ; une fenêtre aussi, jusqu'au premier toucher, où le moteur arrive (environ 184 Ko : `page-engine.js` 22,8 + `holo_engine.js` 4,7 + `holo_engine_bg.wasm` 156,8, mesurés sur la leçon 27). Le jumeau web pèse 1,2 Ko compressé (le corps seul, en Brotli qualité 11). À égalité de compression : la page HoloCode est envoyée en qualité 5 (`server.mjs:392`), 8 171 octets pour la leçon 63 plus 250 octets d'en-têtes ; en qualité 11, elle ferait 7 544 octets (relecture, `curl` puis `zlib` de Node, PC).

## Accessibilité, déterminisme, budgets

- **Accessibilité** : le pincement est aussi un geste d'accessibilité (grossir le texte, WCAG 1.4.4) : sa panne est la plus grave de la piste. Une fenêtre a un nom (WCAG 4.1.2) ; le focus n'est jamais perdu (WCAG 2.4.3) ; un menu et un pli se ferment à Échap et rendent le focus ; un contenu qui apparaît au survol ou au focus doit pouvoir être fermé à Échap et rester tant qu'on le regarde (WCAG 1.4.13 : à vérifier avec `On(…hover)` + `If`, qui ne le garantit pas aujourd'hui) ; jamais Tab, toujours (`ADR-061`). Moins de mouvement : une bulle et un pli s'ouvrent sans animation pour qui le demande.
- **Déterminisme** : les signaux nouveaux (`closed`) passent par l'arbitre comme les autres : mêmes gestes, mêmes valeurs. Déplacer le focus ne change aucune valeur. `When` sur un texte se déclenche au moment où la comparaison devient vraie, comme pour un nombre.
- **Budgets** : `Popover` (à la condition dite en B2) et `Details(group:)` ne demandent pas le moteur (une page reste à environ 8,7 Ko, mesuré pour un pli) ; le moteur léger grossit d'environ 1 Ko en tout (estimation).

## Recette qui peut échouer

| Essai | Ce qui doit se passer | Échec si |
|---|---|---|
| R1. `pincer.sh`, leçons 1 et 9 (PC) | `"erreurs":[]` ; leçon 1 : le moteur arrive ; leçon 9 : `"zoom":"scale(…)"` après le pincement | une erreur, ou pas de zoom (c'est le résultat d'aujourd'hui) |
| R2. Galaxy Z Flip 5 et Flip 3, Chrome, leçon 9 puis `panier.holo` : pincer | la page grossit, puis devient des points ; aucune erreur dans `chrome://inspect` | rien ne bouge |
| R3. Leçon 69, « Calculer la somme » (hors piste) | `ok: true`, le nombre rendu s'affiche | `ok: false` (aujourd'hui) |
| R4. `focus-et-noms.mjs`, leçon 63 | le nœud `dialog` a pour nom « Vider le panier ? » | nom vide (aujourd'hui) |
| R5. La même leçon avec `lang: "en"` | la croix s'appelle « Close » | « Fermer » (aujourd'hui) |
| R6. Fermer la fenêtre par Échap, par ✕, puis par une règle ; `?values` | `closed` part trois fois ; `asked` passe à 1 | une fermeture sans signal |
| R7. Leçon 7 au clavier : Entrée sur « Entrer dans l'atelier », puis « Revenir » | le focus arrive sur « L'atelier », annoncé ; au retour, sur « Entrer dans l'atelier » | `body` (aujourd'hui) |
| R7 bis. `focus-vider.mjs` : `panier.holo`, « + » puis « Vider le panier » au clavier | le focus va au titre le plus proche avant le bouton disparu (« Votre panier ») | `body` (aujourd'hui, mesuré) |
| R8. Menu `Popover` : Tab jusqu'à « Menu », Entrée, Tab, Échap ; puis toucher à côté ; JavaScript coupé | il s'ouvre, le Tab suivant entre dans le menu, Échap ferme et rend le focus ; toucher à côté ferme ; sans JavaScript, tout marche ; aucun `page-engine.js` dans le relevé réseau | il faut le moteur, ou Échap ne ferme pas |
| R9. `Details(group: faq)` : ouvrir le premier, puis le second | le premier se ferme | les deux restent ouverts |
| R10. Choisir « Grand », puis « Petit » | la note apparaît et est annoncée, puis disparaît | rien, ou une annonce à chaque choix |
| R11. TalkBack, parcours F15 du cahier | contrôles nommés, focus visible, aucun piège | un bouton sans nom ou un focus perdu |
| R12. Audit axe-core sur les nouvelles leçons (`moteur/outils/accessibility.mjs`, qui demande d'abord `npm install --no-save playwright axe-core`, `accessibility.mjs:4` : la question en attente) | 0 défaut | un défaut |

R2 et R11 demandent un **vrai téléphone** : pas d'appareil ici.

## Objection

**La meilleure raison de ne pas tout faire :** trois façons d'ouvrir quelque chose (`Details`, `Dialog`, `Popover`), c'est une de trop pour un débutant. Un pli qui sait se fermer (B1) couvre le menu d'un site ; une fenêtre couvre le reste. Ajouter `Popover` gagne des kilo-octets et un menu par-dessus la page, au prix d'un choix de plus à expliquer.

**Autre réserve :** un moteur qui déplace le focus tout seul (E1) peut surprendre s'il le fait trop. La règle doit rester étroite : seulement quand le focus serait perdu.

## Expérience requise

1. Corriger le pincement sur une branche, puis **pincer sur le téléphone de Yocthan** (Flip 5 et Flip 3) : c'est lui qui juge.
2. Construire B1 et B2 sur deux pages jumelles du site de référence, puis demander à Yocthan, et à cinq débutants si possible, lequel ils comprennent sans explication.
3. TalkBack : la fenêtre nommée, le focus après « Explorer l'atelier », le menu ouvert puis fermé.
4. Relever le poids réel des deux menus (B1 et B2) avec `poids.sh` : B2 doit rester sans moteur.
5. Décider avec Yocthan : `Popover` oui ou non ; `toggles:` ou un autre mot ; le refus d'une fenêtre sans titre.

## Mises à jour de documents à prévoir

- **Journal** : la régression du pincement et celle du module, avec leur cause (le remplacement de `touches`), et l'erreur commise (la validation d'`ADR-060` ne les a pas essayés).
- **Guide** : `GUIDE.md:1519` reste juste une fois réparé ; ajouter, après construction, le menu, le groupe de plis, `closed`, le focus tenu par le moteur, `When` sur un texte.
- **Leçons** : « Un menu qui s'ouvre et se ferme », « Un seul pli ouvert », « Quand la fenêtre se ferme » ; corriger la leçon 63 pour montrer le nom de la fenêtre ; une leçon « Réagir à un choix ».
- **`NOMS.md`** : retirer `details, summary, dialog` (`:230`) et les événements (`:237`) de « Pas encore là » ; ajouter `Popover`, `toggles:`, `group:`, `closed` s'ils sont retenus, et dire pourquoi `Menu` n'est pas pris.
- **`COMPARAISON-WEB.md`** : corriger `:105-106` et `:165`.
- **`TABLEAU-WEB.md`** : `:612` à baisser tant que le pincement est cassé, puis à remonter ; `:536` et `:537` à environ 80 % (estimation) ; republier la page en ligne du tableau.
- **`ADR-060`** : une note de correction (comme les autres ADR en ont), sans changer son statut : le statut reste à Yocthan.
- **`AGENTS.md`**, section « État » : la régression et sa correction.

Relu le 2026-10-07 : 13 corrections.
