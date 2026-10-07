# Piste 5 — Des composants avec paramètres

> Statut : EXPLORATION. Avis de Claude, pas une décision.

## Ce que Codex demandait

Sa ligne, dans le texte de l'issue #82 (copie locale non versionnée : `moteur/target/pistes-langage-pour-claude-2026-10-06.md:54`) :

> | **5** | **Composants avec paramètres** : contenu, apparence et actions personnalisables ; état propre à chaque instance. | Réutiliser une carte de produit ou un formulaire sans duplication ni collision de noms. `Part` et `Use` constituent déjà le départ. |

Une **instance** (ou « copie ») est un composant posé dans la page : `ArticleCard(name: Sunrise, …)` est une copie d'`ArticleCard`.

## État vérifié (main, 7a48def, 2026-10-07)

**Franchement : cette piste est presque entièrement faite.** Depuis le texte de Codex, `Part` est devenu `Component`, avec des paramètres, des valeurs par défaut, un emplacement pour du contenu, des signaux, un restylage par le CSS, et des collisions refusées. J'ai mesuré le cas de Codex (la carte de produit) sur le site de référence : il marche avec l'écriture d'aujourd'hui. Il reste cinq manques, plus petits.

Pendant l'exploration, `main` est passé de 7a48def à 1119361 (PR 139, vérification d'un fichier de thème seul) ; `components.rs` n'a pas changé. J'ai relancé les essais : mêmes réponses.

**Ce qui existe**

| Demande de Codex | Ce que fait le moteur | Où |
|---|---|---|
| Paramètres | `Component(name:, params: [title, price])`, posé comme un bloc : `ArticleCard(title: "…", price: 120)` ; chaque paramètre nommé ; une faute refusée avec le bon mot | `moteur/src/components.rs:75-166`, `:387-447`, `:489-505` ; `ADR-050` ; leçon 70 |
| Valeurs par défaut | `params: [title, price: 0]` | `components.rs:92-103`, `:439-444` ; `ADR-056` § 2 ; leçon 73 |
| Contenu | les paramètres dans un texte (`{title}`) ou à la place d'une valeur ; un emplacement `children` | `components.rs:168-196`, `:535-615` ; `ADR-058` ; leçon 75 |
| Apparence | `ArticleCard { … }` vise toutes les copies ; `ArticleCard.promo(…)` une seule ; les variables (`--accent`) se redéfinissent par copie ; plusieurs noms de style par bloc | `components.rs:470-476` ; `flat.rs:367` ; `ADR-050` § 6-7 ; leçon 70 |
| Actions | des signaux émis (`emits: [add]`, `On(Add.tap, emit: add)`) que la page branche (`onAdd: cart.add(1)`) ; ou un paramètre qui reçoit le nom d'une valeur de la page (`qty: sunrise`) | `components.rs:108-115`, `:198-242`, `:404-416` ; `ADR-050` § 4, `ADR-056` § 3 ; leçon 73 |
| État propre à chaque copie | par une valeur de la page donnée à chaque copie (`qty: sunrise`), ou par un champ d'une liste dans `Repeat(over:)` (`fav: item.fav`) ; pas de `state:` dans un composant (refusé, `components.rs:119`) | `ADR-050` (tableau, dernière ligne) ; `ADR-057` ; leçon 74 |
| Collisions de noms | les blocs nommés reçoivent le nom de la copie (`Add` → `AddSunrise`) ; un nom porté deux fois est refusé ; deux composants du même nom refusés ; un paramètre ne peut porter ni un mot du langage ni le nom d'une valeur de la page ; un composant ne se pose pas lui-même ; 8 niveaux, 2 000 copies, 16 paramètres au plus | `components.rs:45-55`, `:283-285`, `:335-344`, `:547-555` ; `rules.rs:106` ; `moteur/src/holo.rs:740`, `:754` |
| Formulaire réutilisé | un `Form` dans un composant, posé deux fois : chaque copie a son formulaire (`EnvoiLivraison`, `EnvoiCadre`) | mesuré ci-dessous |

**Le cas de Codex, mesuré : le catalogue du site de référence.** `exemples/site-reference/catalogue.holo` écrit ses douze cartes une à une (90 lignes). Réécrit avec un composant `Oeuvre` et une répétition `Repeat(items:)` (hors du dépôt : `essais-3-4-5/p5/cat-exact.holo`), il produit **exactement le même HTML**, à la marque du composant près (`holo-c-Oeuvre`), en 41 lignes au lieu de 90, et 2 488 octets au lieu de 3 685.

**Ce qui manque encore** (mesuré)

1. **Changer le contenu d'une copie selon un paramètre donné en clair.** `If(current, is: "catalogue", …)` dans un composant, avec `current` donné en texte : refusé (« une condition s'écrit « If(count, is: 0, children: [ … ]) » », `state.rs:67`). Mais un `If` sur un paramètre marche déjà quand ce paramètre reçoit le nom d'une valeur de la page : `p5-11` et l'exemple existant plus bas s'en servent (`If(recu, is: 1, …)`, `If(fav, is: 1, …)`) ; la condition se décide alors pendant la visite. Mesuré à la relecture : une pastille « Nouveau » sur certaines cartes passe aujourd'hui, avec une valeur de la page par carte (`State(nouveauLever: 1, nouveauPorte: 0)`, `Oeuvre(isNew: nouveauLever)`, `If(isNew, is: 1, …)`) ; la pastille reste alors dans le HTML de chaque carte, cachée par `hidden` (`r5-02`). Un menu peut de même souligner la page où l'on est, avec une valeur numérotée par page (`State(ici: 2)`, un `If` par lien, `r5-03`) ; un texte ne marche pas, car un `If` sur une valeur de la page n'accepte qu'un nombre (« « If(ici, is: …) » attend un nombre entier, ou le nom d'une autre valeur », `r5-01`). `ADR-029`, ligne 79, notait : « Le menu ne peut pas souligner « la page où l'on est » » ; c'est possible à l'écran par ce détour, pas pour un lecteur d'écran : le moteur ne pose jamais `aria-current` (aucune occurrence dans `moteur/src`).
2. **Les styles d'un composant importé ne sont pas isolés.** Si la page écrit le même nom de style qu'un composant importé (`.titre`), celui de la page remplace celui du composant en entier (`moteur/src/holo.rs:781`), sans rien dire. Mesuré : le titre de la carte prend la taille de 40px et la couleur du titre de la page ; et, quand la page n'écrit que la taille, la carte perd aussi son gras et sa couleur (mesuré à la relecture). C'est voulu par `ADR-029` § 4 (« Si la page écrit le même style, c'est le sien qui reste » ; son tableau, ligne 64 : « La page gagne ») et gardé par la correction d'`ADR-050` (lignes 46-50), qui laisse la question ouverte ; Codex recommande d'isoler (`proposals/GPT5.6/web-assez-utilisable-2026-10-07/README.md:445-548`).
3. **L'état propre à chaque copie est verbeux.** Chaque copie demande une valeur de la page, déclarée et nommée à la main. Mesuré : deux formulaires « Question » posés sur une page demandent quatre valeurs (`questionA`, `questionB`, `recuA`, `recuB`). Codex conseille de ne pas ajouter d'état propre dans la première version (même fichier, lignes 337-370).
4. **Un seul emplacement pour du contenu** (`ADR-058`, « Ce qui n'est pas fait ») ; **les noms internes restent publics** (`AddSunrise` est visible de la page, `ADR-056` § 4) ; **un paramètre ne reçoit ni liste ni bloc**, à part `children` (`components.rs:421-423`).
5. **Une faute trouvée dans un composant importé, après sa pose, est mal située** (ajout de la relecture). Elle est rendue sous le nom de la page, avec la ligne du fichier du composant : mesuré, « `c2.holo` : ligne 6, colonne 7 : une condition s'écrit … », alors que la ligne 6 de `c2.holo` est un `Nav` ; le `If` fautif est à la ligne 6 d'`oeuvre2.holo`. Une faute vue dès la lecture du composant (`state:`), elle, est bien attribuée : « « oeuvre.holo » : « Component » n'a pas de réglage « state » ».

**Documents en retard**

- `docs/01-holocode/GUIDE.md:1806` : « Un composant n'a pas d'emplacement pour du contenu » ; faux depuis `ADR-058`. `GUIDE.md:1802` (« Pas de condition sur un champ dans une ligne ») et `:1801` (« l'envoi d'un fichier ») sont aussi dépassés (`ADR-057`, `ADR-059`). `GUIDE.md:1661` : la ligne de `Page` ne cite pas `components` ni `modules`.
- `docs/01-holocode/COMPARAISON-WEB.md:116` : encore `Part(name: Menu, …)`.
- `docs/01-holocode/NOMS.md:102` : « `props` et `slot` ne sont pas repris » ; l'emplacement existe sous le nom `children` (`ADR-058`). `NOMS.md:232` range `template`, `slot` dans « Pas encore là ».
- **Les noms de style de plusieurs mots : un défaut du moteur, pas seulement un document en retard.** La règle est écrite à quatre endroits : `docs/01-holocode/NOMS.md:14` et `GUIDE.md:70` (« comme en CSS : minuscules, mots joints par `-` »), `ADR-037` § 3 (« Les styles gardent l'écriture du CSS »), et le message même du moteur (`moteur/src/holo.rs:551` : « un nom de style s'écrit en minuscules, comme « card » ou « big-card » »). Mais le lecteur refuse le tiret. Mesuré : `P.titre-oeuvre(…)` est refusé à la pose (« caractère inattendu « - » » : un nom de bloc ne lit que lettres, chiffres, `_` et `.`, `holo.rs:213`, `:239`), et `.titre-oeuvre { … }` dans les styles (« après « .titre », une accolade « { » est attendue », `holo.rs:293`, `:405-411`). Le seul nom de plusieurs mots accepté est `titreOeuvre`, qui contredit « en minuscules » (`GUIDE.md:278`, `ADR-050` § 7). Il faut choisir : corriger le lecteur (accepter le tiret promis) ou corriger les quatre textes.
- `exemples/site-reference/catalogue.holo:1-2` : « le langage n'a pas encore de liste répétée » ; c'est dépassé. (Le cahier de Codex voulait douze cartes écrites à la main pour la V1, `proposals/GPT5.6/site-reference-2026-10-06/README.md:35` : la raison du commentaire est fausse, le choix peut rester.)
- `exemples/lecons/70-composants.test:2` : « holo essai … .essai » ; la commande s'appelle `holo test` depuis `ADR-060`.

**Mesures** (`holo check` et `holo test` sur PC ; il n'y a pas de téléphone ici, et rien de cette piste ne dépend de l'appareil)

```bash
H=moteur/target/release/holo.exe          # essais dans essais-3-4-5/p5/
"$H" check - < p5-01-etat-propre.holo                # Component(…, state: State(n: 0), …)
ligne 4, colonne 47 : « Component » n'a pas de réglage « state » ; réglages possibles : name, params, emits, children, rules
"$H" check - < p5-02-if-sur-parametre.holo           # If(current, is: "catalogue", …) dans un composant
ligne 7, colonne 9 : une condition s'écrit « If(count, is: 0, children: [ … ]) » ; comparaisons : is (égal), …
"$H" check - < p5-03-collision-de-noms.holo          # la copie Sunrise de Card (bouton Add) et un Button(name: AddSunrise)
ligne 10, colonne 12 : le nom « AddSunrise » est déjà porté par un autre bloc : deux blocs ne partagent pas un nom
"$H" check - < p5-04-champ-dans-un-composant-en-liste.holo   # ArtCard(fav: item.fav) dans Repeat(over:)
ok
"$H" check - < p5-05-deux-copies-valeurs-de-page.holo        # Counter(n: pommes), Counter(n: poires)
ok
"$H" check p5-07-deux-imports-meme-composant.holo           # deux fichiers importés définissent Card
p5-07-deux-imports-meme-composant.holo : ligne 2, colonne 1 : « b.holo » : deux morceaux importés s'appellent « Card »
"$H" check p5-09-page-et-import-meme-composant.holo
p5-09-page-et-import-meme-composant.holo : ligne 3, colonne 45 : deux composants s'appellent « Card »
"$H" html p5-10-style-de-page-ecrase-le-composant.holo | grep -o '\.holo-s-titre{[^}]*}\|<h2[^>]*>[^<]*</h2>'
.holo-s-titre{font-size:clamp(1.5rem,6.25vw,2.5rem);color:#101020;}
<h2 class="holo-H2 holo-c-Card holo-s-titre">Carte</h2>
"$H" html p5-11-formulaire-deux-fois.holo | grep -o '<form[^>]*>'
<form class="holo-Form" data-name="EnvoiLivraison" novalidate>
<form class="holo-Form" data-name="EnvoiCadre" novalidate>
"$H" check - < p5-12-nom-de-style-tiret.holo         # P.titre-oeuvre("x")
ligne 1, colonne 25 : caractère inattendu « - »
"$H" check - < p5-13-nom-de-style-majuscule.holo     # P.titreOeuvre("x")
ok
```

Le catalogue, comparé octet pour octet :

```bash
"$H" html exemples/site-reference/catalogue.holo > html-original.txt
"$H" html cat-exact.holo > html-composant.txt
sed 's/ holo-c-Oeuvre//g' html-composant.txt > html-composant-sans-marque.txt
cmp -s html-original.txt html-composant-sans-marque.txt && echo "HTML identique (hors la marque holo-c-Oeuvre)"
HTML identique (hors la marque holo-c-Oeuvre)
wc -l -c cat-exact.holo exemples/site-reference/catalogue.holo
  41 2488 cat-exact.holo
  90 3685 exemples/site-reference/catalogue.holo
```

Ajouts de la relecture (fichiers dans `relecture-3-4-5/`) : un `If` sur un paramètre qui reçoit une valeur de la page.

```bash
"$H" check - < r5-01-if-par-valeur-de-page.holo      # State(ici: "catalogue") ; Menu(current: ici) ; If(current, is: "catalogue", …)
ligne 8, colonne 21 : « If(ici, is: …) » attend un nombre entier, ou le nom d'une autre valeur
"$H" check - < r5-02-pastille-par-valeur-de-page.holo   # State(nouveauLever: 1, nouveauPorte: 0) ; Oeuvre(isNew: nouveauLever) ; If(isNew, is: 1, …)
ok
"$H" html r5-02-pastille-par-valeur-de-page.holo | grep -o '<div class="holo-If"[^>]*>'
<div class="holo-If" data-if="nouveauLever|is=1">
<div class="holo-If" data-if="nouveauPorte|is=1" hidden>
"$H" check - < r5-03-menu-par-numero.holo            # State(ici: 2) ; Menu(current: ici) ; un If par lien, avec else:
ok
```

## Le scénario du site de référence

Trois tâches, tirées du site construit (`exemples/site-reference/`) :

1. **Les douze cartes du catalogue** (`catalogue.holo:14-87`) : une carte écrite une fois. **Faisable aujourd'hui** (mesuré ci-dessus).
2. **Le menu commun qui souligne la page où l'on est** (`commun.holo:5-25`, sept liens). Il faut que le menu sache sur quelle page il est posé, et le dise aussi au lecteur d'écran (`aria-current="page"`). **À l'écran, possible aujourd'hui par un détour** : le menu devient un composant à paramètre, chaque page déclare un numéro (`State(ici: 2)`) et le menu écrit chaque lien deux fois, dans un `If` avec `else:` (mesuré : `r5-03`). **Le dire au lecteur d'écran est impossible** : `aria-current` n'existe pas.
3. **Une pastille « Nouveau » sur certaines œuvres seulement, et un bouton « Favori » par carte.** L'accueil pose sa pastille à la main, sur la seule première carte (`accueil.holo:21-24`). Avec un composant, c'est possible aujourd'hui, mais seulement par des valeurs de la page déclarées à la main pour chaque carte, une pour la pastille et une pour le favori (mesuré : `r5-02` pour la pastille, l'exemple existant plus bas pour le favori) ; la pastille cachée reste alors dans le HTML. Une condition sur un paramètre donné en clair (`isNew: true`) est refusée.

Et un cas que Codex cite : **un formulaire réutilisé** (une question sur la livraison, une sur le cadre). Faisable aujourd'hui, avec deux valeurs de page par copie.

## Options comparées

| Manque | Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|---|
| La page où l'on est | A | rien à écrire : un `A` qui mène à la page affichée reçoit `aria-current="page"` ; un état de style `current: { … }` | `aria-current="page"` écrit à la main sur chaque page, puis `a[aria-current=page]` | aucun (le `Navigator` ne le fait pas) | aucun oubli possible ; le lecteur d'écran dit « page actuelle » | le moteur doit connaître le nom du fichier qu'il fabrique (`server.mjs:66` le connaît déjà) |
| La page où l'on est | B | `Menu(current: "catalogue")` et `If(current, is: "catalogue", …)` dans le menu | une variable par page et un `if` dans un gabarit | un paramètre et un `if` dans `build()` | général | chaque page doit le dire ; un oubli ne se voit pas |
| La page où l'on est | C | rien de nouveau : `State(ici: 2)` dans chaque page, `Menu(current: ici)`, et un `If(current, is: 2, …, else: [ … ])` par lien (accepté aujourd'hui, mesuré : `r5-03`) | idem | idem | existe | un numéro à tenir par page ; chaque lien écrit deux fois ; `aria-current` toujours absent |
| Varier le contenu d'une copie | A | `If(isNew, is: true, children: [ … ])` sur un paramètre, décidé au moment où la copie est posée | `{isNew && <Badge/>}` (React), `v-if` (Vue) | `if (isNew) Badge()` dans `build()` | aucun mot nouveau ; rien ne reste dans la page fabriquée | un `If` qui se décide à la lecture et un `If` qui se décide pendant la visite s'écrivent pareil |
| Varier le contenu d'une copie | B | passer le morceau qui change par `children: [ … ]` | `<slot>` | `child:` | existe | verbeux ; chaque page recopie la pastille |
| Varier le contenu d'une copie | C | deux composants (`Oeuvre`, `OeuvreNouvelle`) | deux gabarits | deux widgets | existe | duplication : ce que les composants devaient éviter |
| Varier le contenu d'une copie | D | une valeur de la page par copie (`State(nouveauLever: 1)`, `Oeuvre(isNew: nouveauLever)`) et `If(isNew, is: 1, …)` (accepté aujourd'hui, mesuré : `r5-02`) | un état par carte | un `bool` dans l'état | existe | une valeur déclarée à la main par carte ; la pastille cachée reste dans le HTML ; la condition se décide pendant la visite |
| Isoler les styles | A | rien à écrire : dans un fichier dont la racine est un `Component` à paramètres, un nom de style ne vaut que pour ce composant | `<style scoped>` (Vue), classes générées (Svelte, Angular) | sans objet : le style est dans les paramètres | ce qu'on écrit dans un composant ne fuit pas, et rien ne l'écrase par mégarde | un même nom (`.titre`) veut dire deux choses ; revient sur la correction d'`ADR-050` |
| Isoler les styles | B | refuser la collision : « `.titre` est aussi un style du composant Oeuvre ; renomme l'un des deux » | aucun équivalent | — | simple ; pas de magie | gêne la page qui voulait vraiment ajuster le style du composant |
| Isoler les styles | C | une règle d'écriture : préfixer les noms (`.oeuvreTitre`) | BEM (`.oeuvre__titre`) | — | rien à construire | rien ne l'impose ; la collision reste silencieuse |
| État propre à chaque copie | A | rien de nouveau : une valeur de la page par copie (`fav: favLever`), ou un champ dans une liste | `useState` dans chaque copie (React), `data()` (Vue) | `StatefulWidget` | la page reste seule propriétaire de ses valeurs (l'avis de Codex) | verbeux ; une valeur oubliée ou partagée par erreur se voit mal |
| État propre à chaque copie | B | `Component(state: State(fav: 0))`, déplié en valeurs de la page au nom de la copie (`favLever`, `favPorte`) | idem | idem | court ; un nom stable écrit par l'auteur (celui de la copie) ; l'arbitre ne change pas ; `?values` et `keep` voient des valeurs ordinaires | refusé dans `Repeat(over:)`, où les lignes changent (là, le champ de la liste reste la bonne réponse) |
| État propre à chaque copie | C | un vrai état caché dans chaque copie | idem | idem | — | identité, durée de vie, sauvegarde, jeu à plusieurs : tout ce que Codex énumère ; à refuser |
| Plusieurs emplacements | A | `params: [title, actions, children]`, un paramètre qui reçoit une liste de blocs | `<slot name="actions">` | des paramètres de type `Widget` | général | à construire ; `ADR-058` attend un exemple réel |
| Plusieurs emplacements | B | un seul `children` (aujourd'hui) | — | — | simple | une fenêtre avec un pied à boutons doit tout mettre dans `children` |
| Noms internes publics | A | `exposes: [Add]` : seuls ces noms sont visibles de la page | `defineExpose` (Vue) | une clé (`GlobalKey`) | un composant peut changer sans casser la page | un mot de plus ; les signaux (`emits`) font déjà ce travail |
| Noms internes publics | B | public, comme aujourd'hui (`ADR-056` § 4) | — | — | rien ne casse | une page qui écoute `AddSunrise` casse si le composant change |

**Noms (`ADR-016`)**

- `current:` (état de style) : sur le web, `aria-current="page"` dit la même chose (« la page actuelle »), et Vue Router pose une classe `router-link-active`. CSS a une pseudo-classe `:current`, mais pour autre chose (le passage lu d'une vidéo sous-titrée), presque inconnue. Risque faible.
- `If` sur un paramètre : aucun mot nouveau. Le sens est le même qu'ailleurs (montrer selon une valeur). Le risque : la même écriture se déciderait tantôt à la lecture (un paramètre donné en clair, `isNew: true`), tantôt pendant la visite (un paramètre qui reçoit le nom d'une valeur de la page, ce qui marche déjà : `r5-02`, `p5-11`). Un paramètre donné en clair ne change pas ; une valeur de la page, si. À écrire dans le guide.
- `state:` dans `Component` : le même mot que `Page(state:)`, avec le même sens (« les valeurs que je retiens »). En React (`useState`), en Vue (`data`), en Flutter (`State`), l'état d'une copie est caché à la page. Ici, il deviendrait des valeurs de la page (`favLever`), visibles dans `?values`. Risque moyen : il faut le dire clairement.
- `exposes` (plus tard) : Vue a `defineExpose`, avec le même sens. Risque faible.
- L'isolation des styles n'ajoute aucun mot.

## Recommandation

Dans cet ordre :

1. **Mettre les documents à jour** (le guide § 11, le commentaire du catalogue, l'essai de la leçon 70). Rien à construire. Les noms de style à plusieurs mots, eux, demandent un choix : corriger le lecteur pour accepter le tiret promis par quatre textes, ou corriger ces textes (voir « Documents en retard »).
2. **La page où l'on est, option A** : `aria-current="page"` posé par le moteur, et l'état `current:`. C'est un gain d'accessibilité sans rien écrire, et la seule vraie demande du site de référence.
3. **`If` sur un paramètre, option A**, décidé quand la copie est posée.
4. **Isoler les styles des composants importés, option A**, mais seulement pour un fichier dont la racine est un `Component` à paramètres. Le menu et le thème communs du site de référence (`commun.holo`, un composant sans paramètres, et ses styles `.carte`, `.prix`) restent partagés, comme `ADR-050` l'a voulu. Cela demande l'accord de Yocthan, puisque c'est la question laissée ouverte par la correction d'`ADR-050`.
5. **L'état propre, option B**, seulement après l'expérience ci-dessous. Je suis plus prudent que pour le reste : Codex a de bonnes raisons de dire « pas dans la première version ».
6. Plusieurs emplacements et noms internes privés : attendre un exemple réel, comme `ADR-056` et `ADR-058` le disent.

Pour le site de référence : garder les douze cartes écrites à la main pour la V1 (c'est ce que le cahier teste), corriger seulement le commentaire ; la version par composant peut devenir une leçon ou une page de comparaison.

## Exemple d'auteur

Deux fichiers. « proposé » marque ce qui n'existe pas aujourd'hui.

`oeuvre.holo`, le composant :

```holo
Component(
  name: Oeuvre,
  params: [image, alt, title, price, isNew: false],
  state: State(fav: 0),                                             // proposé : une valeur par copie
  children: [
    Column.carte(gap: 6px, children: [
      If(isNew, is: true, children: [ Text.pastille("Nouveau") ]),  // proposé : If sur un paramètre donné en clair
      Image(source: image, alt: alt),
      P.titre("{title}"),
      Text("{price} euros"),
      Button(name: Fav, text: "Favori"),
      If(fav, is: 1, children: [ Text("Dans vos favoris") ]),
    ]),
  ],
  rules: [ On(Fav.tap, effect: fav.set(1)) ],
)

// proposé : dans ce fichier, ces noms ne valent que pour Oeuvre
.carte { background: #fffaf3; border-radius: 14px; padding: 12px; }
.titre { font-weight: bold; color: #8a3b12; }
.pastille { background: #c2462b; color: #fffaf3; border-radius: 999px; padding: 3px 10px; }
```

`catalogue.holo`, la page :

```holo
import "oeuvre.holo"

Page(
  title: "Catalogue",
  children: [
    Nav(children: [ Row(gap: 16px, children: [ A("Accueil", to: "accueil.holo"), A("Catalogue", to: "catalogue.holo") ]) ]),
    H1.titre("Catalogue"),
    Grid(columns: 2, gap: 16px, children: [
      Oeuvre(name: Lever, image: "lever.svg", alt: "Un soleil jaune se lève au-dessus d'un fleuve bleu", title: "Lever sur le fleuve", price: 120, isNew: true),
      Oeuvre(name: Porte, image: "porte.svg", alt: "Une porte bleue dans un mur couleur sable", title: "La porte bleue", price: 90),
    ]),
  ],
)

A { current: { font-weight: bold; text-decoration: underline; } }   // proposé : l'état « current »
.titre { font-size: 40px; }                                          // le .titre de la page, qui ne touche plus les cartes
```

Vérifié avec le moteur actuel : la version proposée est refusée à chacune des trois lignes « proposé » qui ajoutent un mot (`current:`, `state:`, `If` sur un paramètre donné en clair). La quatrième (l'isolation des styles) passe la vérification, mais donne le mauvais résultat : le `.titre` de la page remplace celui des cartes (mesuré à la relecture : il ne reste que la taille ; le gras et la couleur des cartes sont perdus). La même chose écrite avec l'existant passe, au prix de deux valeurs déclarées à la main, d'un nom de style renommé (`titreOeuvre`), sans pastille et sans page courante (la pastille et un menu souligné seraient possibles avec d'autres valeurs de la page, mesuré : `r5-02`, `r5-03`) :

```text
"$H" check essais-3-4-5/p5/exemple-propose/catalogue.holo
ligne 15, colonne 5 : « current » n'est pas un état ; un style décrit ces états : hover, focus, active, dark, phone (ADR-036)
(sans current :) « oeuvre.holo » : « Component » n'a pas de réglage « state » ; …
(sans current ni state :) ligne 6, colonne 7 : une condition s'écrit « If(count, is: 0, children: [ … ]) » ; …
                          # rendu sous le nom de la page, mais la ligne 6 est celle d'oeuvre2.holo (manque 5)
"$H" check essais-3-4-5/p5/exemple-existant/catalogue.holo
ok
"$H" test essais-3-4-5/p5/exemple-existant/catalogue.holo essais-3-4-5/p5/exemple-existant/catalogue.test
catalogue.test : ok, 3 ligne(s) jouée(s)        # tap FavLever ; expect favLever = 1 ; expect favPorte = 0
```

Le même, en HTML, CSS et JavaScript. Il fait la même chose : la page courante marquée, la pastille sur la première carte seulement, un « Favori » par carte, des styles de carte qui ne fuient pas, la page lisible sans JavaScript (seul le bouton en a besoin). Relu : la première version du jumeau oubliait ce que le moteur ajoute seul (la colonne de 640 px, les 16 px entre les blocs, la rangée du menu, l'allure d'un bouton) ; ces six lignes sont ajoutées, sous le commentaire « Ce que le moteur ajoute seul ».

```html
<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Catalogue</title>
<style>
/* Ce que le moteur ajoute seul : la colonne de 640 px, 16 px entre les blocs, la rangée du menu, l'allure d'un bouton. */
body { margin: 0; }
main { display: block; max-width: 640px; margin: 0 auto; }
main > * { display: block; box-sizing: border-box; margin: 0 0 16px; }
nav { display: flex; flex-wrap: wrap; align-items: center; gap: 1rem; }
.grille > * { margin: 0; min-width: 0; box-sizing: border-box; }
.fav { font: inherit; color: inherit; cursor: pointer; background: transparent; border: 1px solid currentColor; border-radius: 6px; padding: 6px 12px; }
nav a[aria-current="page"] { font-weight: bold; text-decoration: underline; }
.titre { font-size: clamp(1.5rem, 6.25vw, 2.5rem); }
/* Les styles de la carte, isolés à la main : chaque règle commence par .oeuvre */
.oeuvre.carte { display: flex; flex-direction: column; gap: 0.375rem; background: #fffaf3; border-radius: 0.875rem; padding: 0.75rem; }
.oeuvre.carte > * { margin: 0; }
.oeuvre .titre { font-size: inherit; font-weight: bold; color: #8a3b12; }
.oeuvre .pastille { background: #c2462b; color: #fffaf3; border-radius: 62.4375rem; padding: 0.1875rem 0.625rem; }
.grille { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fill, minmax(min(100%, max(7.5rem, calc((100% - 1rem) / 2))), 1fr)); }
</style>
</head>
<body>
<main>
  <nav><a href="accueil.html">Accueil</a> <a href="catalogue.html" aria-current="page">Catalogue</a></nav>
  <h1 class="titre">Catalogue</h1>
  <div class="grille">
    <div class="oeuvre carte">
      <span class="pastille">Nouveau</span>
      <img src="lever.svg" alt="Un soleil jaune se lève au-dessus d'un fleuve bleu">
      <p class="titre">Lever sur le fleuve</p>
      <span>120 euros</span>
      <button type="button" class="fav">Favori</button>
      <span class="fav-texte" hidden>Dans vos favoris</span>
    </div>
    <div class="oeuvre carte">
      <img src="porte.svg" alt="Une porte bleue dans un mur couleur sable">
      <p class="titre">La porte bleue</p>
      <span>90 euros</span>
      <button type="button" class="fav">Favori</button>
      <span class="fav-texte" hidden>Dans vos favoris</span>
    </div>
  </div>
</main>
<script>
  // Une valeur par carte : chaque bouton ne montre que le texte de sa propre carte.
  for (const carte of document.querySelectorAll(".oeuvre")) {
    carte.querySelector(".fav").addEventListener("click", () => {
      carte.querySelector(".fav-texte").hidden = false;
    });
  }
</script>
</body>
</html>
```

La différence honnête : en HTML seul, la carte est **écrite deux fois**. Pour l'écrire une fois, il faut un `<template>` et une fonction JavaScript (la page n'est plus lisible sans JavaScript), ou un outil de construction (React, Vue, Svelte). Et `aria-current` s'écrit à la main sur chaque page.

## Par couche

- **Langage** : l'état de style `current:` ; `If` sur un paramètre (aucun mot nouveau) ; `state:` dans `Component` ; l'isolation des styles d'un fichier de composant (aucun mot nouveau, un sens nouveau). Plus tard : un paramètre qui reçoit une liste de blocs, `exposes:`.
- **Moteur** : `components.rs` (décider les `If` sur un paramètre quand la copie est posée ; déplier `state:` en valeurs de la page au nom de la copie, refusé dans `Repeat(over:)`) ; `moteur/src/holo.rs` (retenir de quel fichier vient chaque style : la lecture le sait, `holo.rs:711`, `:722`, `:747`, puis l'oublie, `:782`) ; `flat.rs` (les sélecteurs isolés, par exemple `.holo-c-Oeuvre .holo-s-titre`, du CSS ordinaire que tous les navigateurs lisent, sans `@scope` ; `aria-current` sur un `A` qui mène à la page fabriquée) ; `holo.rs` de la ligne de commande et `flat_view` reçoivent le nom du fichier.
- **Enveloppe navigateur** : `page-engine.js` reprend la page fabriquée par le serveur quand elle existe (`ADR-033`) ; sinon, il la refait lui-même avec le moteur WebAssembly (`page-engine.js:550` : `root.innerHTML = flat_view(source, base, site)`). Il faudra donc lui passer aussi le nom du fichier, sans quoi `aria-current` disparaîtrait de la page refaite. Le panneau `?values` montrera `favLever` comme une valeur ordinaire (rien à changer).
- **Services serveur** : `moteur/outils/server.mjs:66` passe déjà le chemin du fichier à `holo html` ; il suffit que le moteur s'en serve. Plus tard, le serveur proposé (`proposals/Claude/serveur-et-comptes-2026-10-07.md`) gagnerait des noms de valeurs stables pour partager l'état d'une copie.

## Dépendances

- La piste 3 (design) se sert de `current:` pour le style du menu.
- La piste 4 (place disponible) : une carte qui contient une `Row(columnBelow:)` s'adapte dans chaque copie.
- La piste 1 (listes de données) : un catalogue venu du serveur passe par `Repeat(over:)` et les champs d'une liste ; c'est là que l'état d'une ligne vit, pas dans `state:`.
- La piste 6 (interactions) : le bouton « Favori » devrait annoncer son état (`aria-pressed`) ; un `Button` n'a aujourd'hui que `name` et `text` (`GUIDE.md`, aide-mémoire).
- L'isolation des styles revient sur la correction d'`ADR-050` : une décision de Yocthan d'abord.

## Coût

- **Moteur** : environ 400 à 450 lignes de Rust, essais compris (estimation) : la page où l'on est, environ 80 ; `If` sur un paramètre, environ 60 ; l'isolation des styles, environ 120 ; `state:` déplié, environ 200 (estimations).
- **Poids transféré** : rien en JavaScript. `aria-current` ajoute une vingtaine d'octets à une page ; un sélecteur isolé, une vingtaine d'octets par règle (estimations). Les composants sont dépliés avant d'arriver au navigateur : pas de moteur de composants à télécharger (vérifié : le catalogue par composant donne le même HTML, mesuré).
- **Travail** : 3 séances, avec deux leçons (la page où l'on est ; une copie qui change selon un paramètre), et une troisième pour `state:` s'il est retenu (estimation).

## Accessibilité, déterminisme, budgets

- **Accessibilité** : `aria-current="page"` est annoncé par les lecteurs d'écran (« page actuelle ») : il ne peut plus être oublié. Un composant hérite de l'accessibilité de ses blocs (un vrai `<button>`, un `alt` obligatoire, une étiquette obligatoire) : vérifié par les tests du moteur et l'audit des leçons (`ADR-055`, `ADR-058`). Le « Favori » n'annonce pas encore son état : à prévoir avec la piste 6.
- **Déterminisme** : tout se décide au dépliage, avant d'envoyer la page. Même fichier, même nom de fichier, même HTML. L'état déplié porte des noms stables, écrits par l'auteur (le nom de la copie) : la même suite de gestes donne les mêmes valeurs (`holo test` le vérifie déjà).
- **Budgets** : 16 paramètres, 8 niveaux, 2 000 copies au plus (`components.rs:45-49`) ; `state:` ajouterait des valeurs à la page, qu'il faudra compter dans ses limites. Aucune mesure de téléphone : pas d'appareil ici, et rien ne tourne de plus pendant la visite.

## Recette qui peut échouer

1. **La page où l'on est** : `holo html catalogue.holo`, puis `holo html accueil.holo`, avec le même menu importé. Doit : dans chaque page, exactement un `aria-current="page"`, sur le lien qui mène à cette page même. Échec : aucun (aujourd'hui, mesuré), ou deux dans une page, ou le mauvais lien marqué.
2. **`If` sur un paramètre** : `Oeuvre(isNew: true)` et `Oeuvre()`. Doit : la pastille dans le HTML de la première, **absente** (pas cachée) du HTML de la seconde. Échec : refus (aujourd'hui, mesuré) ou une pastille cachée par `hidden`.
3. **Styles isolés** : le cas `p5-10` (la page écrit `.titre` à 40px). Doit : le titre de la carte garde la couleur `#8a3b12` et sa taille. Échec : il prend 40px et `#101020` (aujourd'hui, mesuré). Et le menu commun du site de référence garde `.carte` et `.prix` partagés : `holo html` du site identique avant et après.
4. **État propre** : deux `Question` sans valeurs déclarées. `holo test` : `tap EnvoyerLivraison`, puis `expect recuLivraison = 1` et `expect recuCadre = 0`. Échec : une valeur partagée, ou un nom qui change d'une fois à l'autre. Et `state:` dans un composant posé par `Repeat(over:)` : refusé, avec « utilise un champ de la liste ».
5. **Collision créée par l'état** : la page déclare déjà `favLever` et pose `Oeuvre(name: Lever)` avec `state: State(fav: 0)`. Doit : refusé, avec les deux noms. Échec : une valeur écrasée sans rien dire.
6. **Rien ne change pour l'existant** : `holo html` des 81 leçons et du site de référence, avant et après. Doit : identique octet pour octet (sauf `aria-current` là où un lien mène à la page elle-même, à lister).

## Objection

**Les composants sont déjà le point fort récent (88 sur 100 dans `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md:472`), et chaque ajout rapproche HoloCode d'un framework**, avec ses pièges : un `If` qui se décide à la lecture et un autre pendant la visite, des styles qui valent ici mais pas là, des valeurs qui naissent du nom d'une copie. Le site de référence n'a qu'un vrai besoin (la page où l'on est), et il se règle sans toucher aux composants. Yocthan veut avancer vers la 3D ; l'idée gardée au chaud de séparer un jour le langage, le socle et un framework de composants (`proposals/Claude/idees/langage-socle-framework-2026-10-07.md`) dit aussi d'attendre. Je retiens l'objection pour l'état propre (option B), que je ne recommande qu'après une expérience ; pas pour la page où l'on est, qui est un gain d'accessibilité.

## Expérience requise

- **Avec deux ou trois débutants** (ceux de la recette U01-U03, quand ils seront trouvés) : ajouter une pastille « Nouveau » à une œuvre et souligner la page courante dans le menu, avec le guide seul. Compter les blocages. Puis le même travail avec l'écriture proposée.
- **Le formulaire réutilisé** : écrire la page des deux « Question » avec des valeurs de page, puis avec `state:` (en prototype hors du dépôt). Compter les lignes et les erreurs faites en route. Si l'écart est faible, garder l'option A.
- **Les collisions réelles** : importer ensemble des composants de plusieurs leçons et du site de référence, et compter les noms de style en commun. Si aucun, l'isolation peut attendre.
- **Le lecteur d'écran** : vérifier que TalkBack annonce « page actuelle » sur le téléphone de Yocthan. Pas d'appareil ici.

## Mises à jour de documents à prévoir

- `docs/01-holocode/GUIDE.md` § 6 sexies bis : `If` sur un paramètre, l'isolation des styles, `state:` (si retenu), l'état `current:` ; § 5 : `current:` parmi les états, la règle des noms de style de plusieurs mots ; § 10 : la ligne de `Page` (`components`, `modules`) ; § 11 : retirer les trois lignes dépassées.
- Leçons nouvelles dans `exemples/lecons/` : la page où l'on est ; une copie qui change selon un paramètre (numéros à choisir après 81). Leçon 70 : corriger son essai (`holo test`, `.test`).
- `exemples/site-reference/catalogue.holo` : corriger le commentaire des lignes 1-2.
- `docs/01-holocode/NOMS.md` : ligne 102 (l'emplacement `children`), ligne 232, ligne 14 (les noms de style de plusieurs mots, à accorder avec `GUIDE.md:70` et `:278`, `ADR-037` § 3 et le message de `holo.rs:551`), `current`, `state:` dans `Component`.
- `docs/01-holocode/COMPARAISON-WEB.md` : ligne 116 (`Component`), une ligne pour `aria-current`.
- `docs/01-holocode/TABLEAU-WEB.md` : ligne 546 (les composants), une ligne pour la page courante.
- `docs/02-gouvernance/adr/ADR-029` (ligne 79) et `ADR-050` (correction, lignes 46-50) : ce qui change, si Yocthan le décide.
- `docs/06-journal/JOURNAL.md` : l'entrée de l'étape.

Relu le 2026-10-07 : 20 corrections.
