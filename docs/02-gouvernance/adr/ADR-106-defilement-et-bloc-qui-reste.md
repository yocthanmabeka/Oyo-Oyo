# ADR-106 — Où en est le visiteur dans la page, et un bloc qui reste à l'écran : `scroll`, `sticky: top | bottom`

- Statut : ACCEPTÉ (fait et validé : Yocthan, 2026-10-09, « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #235 (« Dernière dette du web : la position du défilement comme valeur »), dans la file des dernières dettes du web ouverte par Yocthan le 2026-10-09 ; le grand tableau du web (`docs/01-holocode/TABLEAU-WEB.md` : « défilement (scroll) … En partie … Pas encore la position du défilement comme valeur ») ; `docs/01-holocode/COMPARAISON-WEB.md` (« Approche, position du défilement : manque ») ; la piste 6 de l'exploration du web complet (`proposals/Claude/exploration-web-complet-2026-10/piste-06.md`, options G2 `{scroll}` et G3 `Header(stick: true)`) ; `ADR-017` (un style ne dit que l'apparence : `position` refusé) ; `ADR-035` (refuser une mécanique, jamais une capacité) ; `ADR-061` (`Enter(…, inView: true)`, sans écouter le défilement) ; `ADR-039` et `ADR-081` (des valeurs données par le moteur ou par le serveur, lues sans être changées) ; la règle de parité de Yocthan (2026-10-07 : ce qui existe sur l'ordinateur existe sur le téléphone, et l'inverse).
- Validation : faite par Yocthan le 2026-10-09 (« tu le valides déjà, tu le fais déjà »).
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Deux besoins courants d'une longue page : une **barre de lecture** qui se remplit pendant qu'on lit, et un bouton **« Retour en haut »** qui n'apparaît qu'après avoir descendu. HoloCode savait faire apparaître un bloc quand il arrive à l'écran (`ADR-061`), pas dire où l'on en est.
- Un troisième, plus courant encore : **un en-tête, une barre ou un bouton qui reste à l'écran** pendant qu'on défile. Impossible jusqu'ici : `position` est refusé dans un style (`ADR-017`), et la piste 6 l'avait laissé en attente (G3).
- **Le web le fait ainsi**, avec des défauts connus :
  - en JavaScript, on écoute l'événement `scroll`, qui part à chaque image, et l'on calcule soi-même `scrollY / (scrollHeight - innerHeight)` ; sans frein, la page ralentit et la batterie fond ; en pixels, la valeur ne dit rien d'un écran à l'autre ;
  - en CSS, `position: sticky; top: 0`. Sans `top`, il ne fait rien, sans rien dire. Sous un ancêtre en `overflow`, il ne fait rien non plus (le défaut le plus signalé). Il ne reste qu'à l'intérieur de son parent : dans un parent court, il ne reste jamais. Deux blocs collés au même bord se recouvrent, sauf décalages calculés à la main. Et c'est la guerre des `z-index` ;
  - **un en-tête collé cache l'élément qui a le focus** : on avance avec Tab, et le lien suivant passe dessous. WCAG 2.2 en a fait un critère (2.4.11, « focus non masqué », niveau AA). Le remède, `scroll-padding-top`, est rarement écrit, et doit suivre à la main la hauteur de l'en-tête ;
  - **sur un téléphone**, un grand en-tête collé mange l'écran : téléphone couché, page grossie à 200 % ou 400 % (WCAG 1.4.10), clavier de l'écran ouvert sur un champ ;
  - les animations liées au défilement du CSS (`animation-timeline: scroll()`) donnent une barre sans JavaScript, mais pas encore dans tous les navigateurs (`ADR-061` l'avait déjà écarté pour `view()`), et seulement pour l'allure : ni texte, ni règle.
- En Flutter, `ScrollController` et son `offset` en pixels, écouté par `addListener` ; `SliverAppBar(pinned: true)` pour un en-tête qui reste.

## Décision

1. **`scroll`** : une valeur que le moteur donne, de 0 (tout en haut de la page) à 100 (tout en bas), en pour cent entiers ; 0 pour une page qui tient dans l'écran. La page la lit comme ses autres valeurs : `{scroll}` dans un texte, `If(scroll, over: 10, …)`, `When(scroll, over: 89, effect: …)`, `Progress(value: scroll, max: 100, …)`, une place sur un plateau, `best.set(scroll)`.
2. **On la lit, on ne la change jamais.** Refusé, avec la raison : la déclarer (`State`, `Shared`), la changer (une demande, un champ, un module, l'appareil, un fichier importé, un glissement, un chronomètre), la garder (`keep`), la mettre dans l'adresse (`address:`), la lire dans un monde seul. Des données reçues (`Data`, un module) ne la changent pas : seul le navigateur la donne.
3. **Au plus dix fois par seconde, et seulement si la page la lit.** Le navigateur suit le défilement sans le gêner (un écouteur passif) et donne la place au moteur cent millisecondes au plus tôt après la précédente ; la dernière place est toujours donnée, même quand le visiteur s'arrête entre deux battements. Un pour cent entier ne change pas à chaque pixel. Une page qui lit `scroll` fait venir le moteur tout de suite (`data-live`) ; les autres ne paient rien.
4. **Sans JavaScript**, `scroll` vaut 0 : la page arrive en haut ; ce qui dépend d'elle reste à son départ (une barre vide, pas de bouton « Retour en haut ») ; la page se lit tout entière.
5. **`sticky: top`** ou **`sticky: bottom`**, sur un bloc posé directement dans la page (`Page(children:)`, `Main(children:)`, au besoin sous un `If` posé là), ou sur `Header` et `Footer` posés dans la page : le bloc reste en haut ou en bas de l'écran pendant qu'on défile. Il garde sa place dans la page et dans l'ordre de lecture : c'est le `position: sticky` du CSS, posé par le moteur sur la propre balise du bloc, sans enveloppe. Du CSS seulement : il marche sans JavaScript.
6. **Une exception étroite au refus de `position` (`ADR-017`).** Un style ne dit toujours que l'apparence : `position: sticky` et `position: fixed` y sont refusés, avec le bon mot. Le réglage ne prend que deux bords ; ni décalage (`top: 40px`), ni ordre de superposition (`z-index`), ni bloc fixé qui sort du fil de la page (`fixed`), ni place libre (`absolute`). Un bloc par bord : on range ensemble ce qui doit rester, `Row(sticky: top, children: [ … ])`. La raison (`ADR-035`) : la capacité, une barre ou un en-tête qui reste à l'écran, est courante et manquait ; la mécanique refusée le reste (superposer, décaler, sortir du fil). Le moteur garantit ce que le CSS laisse à l'auteur : un bord, aucun ancêtre en `overflow`, un fond, la marge du focus, la part de l'écran.
7. **Il ne cache jamais ce qui a le focus** (WCAG 2.2, 2.4.11). Le moteur ajoute `scroll-padding-top` et `scroll-padding-bottom`, égaux à la hauteur du bloc, plus 8 pixels. La page mesure le bloc, et suit ses changements (un `If` qui le montre ou le cache, un texte plus grand). Sans JavaScript, la marge vaut le plus grand bloc permis, le cinquième de l'écran. Tab, Maj + Tab et un lien vers un endroit de la page (`A(to: "#Quand")`) s'arrêtent sous la barre, au-dessus du bloc du bas. Un bloc qui apparaît (un `If` devenu vrai en descendant) sur l'élément qui a le focus : ce qu'on voyait revient au-dessus de lui ; ce qu'on a laissé hors de l'écran en défilant y reste.
8. **Le téléphone.**
   - Un bloc qui reste prend au plus le cinquième de la hauteur de l'écran (`max-height: 20svh`, bords compris : 156 pixels sur un écran de 360 × 780) ; ce qui dépasse défile dans le bloc. Avec un bloc en haut et un en bas, il reste toujours plus de la moitié de l'écran.
   - Sous 480 pixels de haut (un téléphone couché, une page grossie à 200 % ou plus), rien ne reste : le bloc reprend sa place dans la page.
   - Pendant qu'on écrit avec le clavier de l'écran, le bloc reprend aussi sa place, pour ne pas cacher le champ. Le clavier se reconnaît à la hauteur visible de l'écran (`visualViewport`), quand un champ a le focus ; un pincement qui grossit la page n'est pas le clavier.
   - Sur papier, le bloc garde sa place.
9. **Les outils du moteur lui font place.** Le bouton rond ☰ monte au-dessus d'un bloc resté en bas. Un bloc du bas se pose au-dessus des touches à l'écran (`ADR-069`).
10. **Un fond d'office.** Sans style, un bloc qui reste prend le fond de la page (sa couleur, ou sa variable, aussi dans le thème sombre), pour que le texte qui passe dessous ne se lise pas à travers. Sous un fond de page en dégradé ou en image, il prend le fond et le texte du navigateur (`Canvas`, `CanvasText`), toujours lisibles ensemble. Le style de l'auteur l'emporte (`.bar { background: #1a1a2e; }`).
11. **Refusés, avec la raison** : un autre bord (`sticky: middle`, `"top"`, `true`) ; un bloc rangé dans un autre (`Row`, `Header`, un monde) ; les blocs qui ne se voient pas ou ont déjà leur place (`If`, `Repeat`, `Dialog`, `Main`, `Point`, `Scenes`…) ; un son sans lecteur (un lecteur de son, avec `label:`, peut rester en bas) ; deux blocs au même bord ; `sticky:` à l'appel d'un composant (il s'écrit sur son bloc racine) ; `position: sticky` ou `fixed` dans un style.
12. **Un essai écrit fait défiler la page** : `scroll 50` dans un fichier `.test` (`holo test`, `ADR-054`).

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le nom de la place | `scrollY`, le mot de JavaScript ; `progress` ; `reading` ; `scrolled` ; `scrollProgress` ; **`scroll`** | `scroll` est le plus court, et c'est le mot du web pour le défilement ; le CSS l'emploie déjà pour sa progression, de 0 % à 100 % (`animation-timeline: scroll()`) : le même sens (`ADR-016`). `scrollY` garderait son sens de pixels, qui ne disent rien d'un écran à l'autre ; `progress` se confondrait avec le bloc `Progress` ; `reading` ne vaut que pour un article ; `scrollProgress` est long. Une valeur s'écrit en minuscules, comme `hour` (`ADR-037`) |
| Son unité | des pixels ; une fraction de 0 à 1 ; **un pour cent entier, de 0 à 100** | les valeurs de HoloCode sont des entiers bornés. Un pour cent se lit d'un coup d'œil (« 42 % »), se compare (`over: 10`), remplit une barre (`max: 100`), et ne change pas à chaque pixel |
| Une valeur ou un signal | `On(Page.scrolled, …)` ; `On(Section.seen, …)` ; **une valeur** | une valeur sert partout où l'on lit : un texte, une condition, une barre, une règle qui guette. Un signal à chaque mouvement ferait courir des règles sans fin. Ce qui doit arriver à un moment s'écrit déjà : `When(scroll, over: 89, …)` |
| Son rythme | à chaque événement, comme JavaScript, souvent sans frein ; à chaque image ; **au plus dix fois par seconde, la dernière place toujours donnée** | soixante fois par seconde réveillent l'arbitre et la page entière : le téléphone ralentit. Dix fois suffisent à l'œil pour une barre, et la dernière place n'est jamais perdue. Rien pour une page qui ne la lit pas |
| Sans JavaScript | une barre en CSS (`animation-timeline: scroll()`) ; **0** | le CSS n'est pas encore dans tous les navigateurs, et ne ferait marcher que la barre, ni le texte ni les règles ; 0 est la vérité d'une page qui arrive : en haut. La barre en CSS reste une amélioration possible |
| Le nom du réglage qui garde à l'écran | `pinned: true` (Flutter, `SliverAppBar`) ; `stick: true` (piste 6) ; `fixed:` ; `stay:` ; un bloc `Sticky(children: …)` ; **`sticky: top \| bottom`** | `sticky` est le mot du CSS, avec son sens : le bloc garde sa place, et reste quand on défile (`ADR-016`). Le bord est dans la valeur, ce que `pinned: true` ne dit pas (en haut, ou en bas ?). `fixed` dit autre chose en CSS : un bloc sorti du fil de la page, posé par-dessus. Un bloc de plus serait plus long, et changerait la place du contenu |
| Où l'écrire | dans un style (`position: sticky; top: 0`) ; sur n'importe quel bloc ; **sur un bloc posé directement dans la page, ou sur `Header` et `Footer`** | un style ne dit que l'apparence (`ADR-017`). Sur n'importe quel bloc, il ne resterait qu'à l'intérieur de son parent (une ligne, une carte), donc pas du tout : le défaut le plus connu de `position: sticky`, avec l'ancêtre en `overflow`. Directement dans la page, il reste toujours, et rien ne l'en empêche |
| Ce qu'on peut régler | un décalage (`top: 40px`), l'ordre de superposition (`z-index`), plusieurs blocs par bord ; **un bord, un bloc par bord** | deux blocs collés au même bord se recouvrent, sauf décalages calculés à la main ; l'ordre de superposition, c'est la guerre des `z-index`. On range ensemble ce qui doit rester |
| Le focus | rien, le défaut du web ; `scroll-padding` écrit par l'auteur ; **la marge mesurée par la page, et un repli en CSS** | WCAG 2.4.11 (AA) : ce qu'on atteint au clavier n'est pas caché. Écrit à la main, `scroll-padding` se désaccorde dès que l'en-tête change de hauteur ; ici, la page le mesure, et sans JavaScript la marge vaut le plus grand bloc permis |
| La part de l'écran | rien ; un refus à la vérification ; **la hauteur bornée au cinquième de l'écran, et rien sous 480 pixels de haut** | la hauteur d'un bloc dépend de l'écran et du texte du visiteur : on ne la connaît pas en vérifiant le fichier. Bornée, elle ne mange jamais l'écran d'un téléphone ; un écran trop bas (couché, grossi) n'a pas la place de garder un bloc |
| Le clavier de l'écran | rien ; **le bloc reprend sa place pendant qu'on écrit** | le clavier prend près de la moitié de l'écran : un en-tête collé cacherait le champ. Rien ne saute : le bloc n'avait jamais quitté sa place dans le fil de la page |

## Les défauts du web évités

- **`position: sticky` qui ne fait rien** : sans bord, sous un ancêtre en `overflow`, dans un parent trop court. Ici, le bord est la valeur, et la place est vérifiée ; le moteur ne pose aucun `overflow` au-dessus de la page.
- **La guerre des `z-index`** : le moteur pose l'ordre, au-dessus de la page, sous ses propres outils (☰, le carrefour).
- **L'en-tête collé qui cache le focus** (WCAG 2.4.11), et le lien vers un endroit qui finit sous l'en-tête : `scroll-padding`, mesuré.
- **Le texte qui se lit à travers** un bloc sans fond : un fond opaque d'office, celui de la page.
- **L'écouteur `scroll` sans frein**, à chaque image, et le calcul en pixels refait à la main : une valeur en pour cent, au plus dix fois par seconde, seulement pour une page qui la lit.
- **L'en-tête qui mange l'écran** d'un téléphone, couché, grossi (WCAG 1.4.10) ou sous le clavier : le cinquième au plus, rien sous 480 pixels de haut, rien pendant qu'on écrit.
- **`position: fixed`**, qui sort le bloc du fil de la page et le pose par-dessus le texte : refusé. Le bloc qui reste garde sa place dans l'ordre de lecture, et le lecteur d'écran le lit où il est écrit.

## Dettes

- Sans JavaScript, `scroll` vaut 0 : une barre en CSS (`animation-timeline: scroll()`) pourra prendre le relais quand tous les navigateurs l'auront.
- Seulement la place dans la page : pas celle d'un bloc qui défile lui-même (`overflow: auto`), ni le défilement de côté.
- Le cinquième de l'écran et le seuil de 480 pixels ne se règlent pas.
- Un `If` qui pose un bloc en haut dans chacune de ses deux branches est refusé (deux blocs au même bord), même si un seul se montre à la fois.
- Un composant ne reçoit pas `sticky:` à l'appel : on l'écrit sur son bloc racine, et on le pose directement dans la page.
- Pour un membre connecté (`ADR-081`) sur une page qui partage des valeurs : après un toucher partagé, la place revient à celle que garde le serveur (0) jusqu'au défilement suivant.
- Le clavier de l'écran est reconnu à la hauteur visible de l'écran ; dans Chrome, l'essai imite cette hauteur : rien n'est encore essayé sur le téléphone de Yocthan, ni avec TalkBack.
- Le bouton rond du moteur monte au-dessus d'un bloc du bas ; le message court du moteur (« Le serveur n'a pas répondu… ») ne monte pas encore.

## Critères de validation

- Tests du moteur (`moteur/src/scroll.rs`) :
  - `scroll_is_given_by_the_browser_and_watched_by_the_rules` : la valeur posée à 0 et la page vivante ; 50, puis 250 bornée à 100 ; la règle qui guette se déclenche une fois ; le titre suit ; remonter ne défait rien ; un toucher ne la change pas ; une page qui ne la lit pas ne la reçoit pas ; des données reçues ne la changent pas ; `holo test` défile ; une page sans `scroll` ni `sticky` n'a rien de plus ;
  - `a_sticky_block_stays_on_screen_and_never_hides_the_focus` : la marque sur la balise du bloc (`Row`, `A`, `Header`, `Footer`, un bloc qui bouge, la racine d'un composant) ; le style (l'écran assez haut, le cinquième, la marge du focus, le bouton du moteur, le clavier) ; le fond de la page, aussi sombre, ou celui du navigateur sous un dégradé ; une page qui ne fait que coller n'est pas vivante ; un lecteur de son en bas ; un monde seul refusé ; `position: sticky` refusé dans un style ;
  - `what_is_refused` : vingt refus, chacun avec sa raison.
- Dans Chrome : « une barre de lecture et un retour en haut … (leçon 129, défilement) ».
  - Sur un ordinateur : la barre vide au départ, nommée « Lecture » pour le lecteur d'écran ; à mi-page, collée en haut, à 50 % ; le bouton collé en bas ; le bouton ☰ au-dessus de lui ; cinquante pas de défilement en une seconde donnent la place dix fois au moteur, et la dernière.
  - « Retour en haut » ramène en haut ; « Quand les voir » pose son titre sous la barre.
  - Tab jusqu'en bas, puis Maj + Tab jusqu'en haut : aucun lien caché. Un lien posé sous la barre, puis sous le bouton du bas, est ramené dans la marge par Tab ; un bouton qui apparaît sur le lien qui a le focus le laisse visible.
  - Sur un téléphone de 360 × 780 : la barre fait 55 pixels ; un en-tête beaucoup trop haut est borné à 156 pixels et défile dedans ; le clavier de l'écran (imité) lui rend sa place, refermé il reste de nouveau. Couché (780 × 360) ou grossi à 200 % (360 × 400) : rien ne reste, la barre part avec la page quand on descend, aucune marge du focus ne reste posée ; sur papier (le média « print » émulé), elle garde sa place, et reste de nouveau à l'écran.
  - Sans JavaScript : la barre reste en haut, vide, sans bouton, et la marge du focus vaut 148 pixels.
  - L'essai rate avec le moteur de `main` (« « Row » n'a pas de paramètre « sticky » ») ; sans le frein (49 envois pour 50 pas) ; sans la marge du focus (le titre sous la barre, le lien sous le bouton) ; sans le retour du focus (le lien reste sous le bouton qui apparaît).
- Leçon `129-une-barre-de-lecture.holo`, et son essai écrit `129-une-barre-de-lecture.test`.
