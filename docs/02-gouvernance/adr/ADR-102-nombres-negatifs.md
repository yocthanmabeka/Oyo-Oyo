# ADR-102 — Des nombres négatifs : `negative: [temperature]`

- Statut : ACCEPTÉ (fait et validé : Yocthan, 2026-10-09, « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #231 (« Dernière dette du web : les nombres négatifs »), ajoutée à la file à la demande de Yocthan le 2026-10-09 ; le grand tableau du web, où les variables étaient « en partie » ; `ADR-023` (une valeur ne descend jamais sous 0) ; `ADR-043` (multiplier, diviser, les formats) ; `ADR-066` (les nombres à virgule, qui annonçait : « les nombres négatifs ne sont pas encore là »).
- Validation : faite par Yocthan (voir le statut).
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Une valeur de la page allait de 0 à un milliard. `sub` s'arrêtait à 0, et `State(t: -5)` était refusé. Une température, un solde, une position dans un jeu, l'écart entre deux scores étaient impossibles.
- Ce garde-fou a une raison : un panier à −1 article est un défaut classique du web, qu'un compteur JavaScript produit dès qu'on touche « Retirer » une fois de trop. Il faut garder ce garde-fou pour les valeurs qui comptent des choses, et l'ouvrir pour celles qui peuvent vraiment passer sous zéro.

## Décision

1. **Une valeur ne descend sous zéro que si la page le dit** : `negative: [balance, temperature]`, sur la page, comme `keep:`. Les autres valeurs s'arrêtent à 0, comme avant : aucune page existante ne change.
2. **Une telle valeur va de −1 000 000 000 à 1 000 000 000**, à son échelle (`ADR-066`). Elle peut partir de sous zéro, `State(temperature: -2)`, `State(balance: -12.50)` ; un départ sous zéro sans `negative:` est refusé, avec le mot à écrire.
3. **Les demandes** :
   - `sub` passe sous zéro ; `set(-10)` fixe un nombre négatif ; `mul(-1)` et `div(-2)` changent le signe (`speed.mul(-1)` : la balle rebondit) ;
   - `add(-5)` s'écrit `sub(5)`, et `sub(-5)` s'écrit `add(5)` : refusés avec le bon mot ;
   - une autre valeur peut être négative : `x.add(speed)` retire quand `speed` vaut −3. Une valeur qui ne peut pas être négative reçoit au plus 0 : `cart.add(gap)`, avec `gap` à −7, ne donne jamais −2 ;
   - **un nombre négatif se calcule comme sans son signe** : −7 ÷ 2 = −3 (comme 7 ÷ 2 = 3), et un arrondi met la moitié du côté opposé à zéro (−14,025 → −14,03, comme 14,025 → 14,03). Le calcul se fait en nombres entiers, exact (`ADR-066`).
4. **Les comparaisons** prennent un nombre négatif, entier ou à virgule : `If(temperature, under: 0)`, `If(temperature, under: -20)`, `When(balance, under: -100, effect: …)`. Comparer une valeur qui ne peut pas être négative à un nombre négatif est refusé (la condition ne serait jamais vraie).
5. **L'affichage met le signe moins de la langue de la page**, celui du CLDR que suit `Intl.NumberFormat` :
   - « -2 » en français, en anglais, en allemand ; « −2 » (U+2212) en suédois, finnois, norvégien, estonien, lituanien, slovène, croate ; en arabe, en hébreu et en persan, une marque de gauche à droite garde le signe devant le nombre au milieu d'une phrase écrite de droite à gauche ;
   - les formats suivent : `{balance:number}` « -1 234 », `{balance:cents}` « -12,50 », `{t:00}` « -05 », et un nombre à virgule, « -12,50 » ;
   - jamais « -0 » : un nombre entier n'a pas de zéro négatif ;
   - le titre de l'onglet (`ADR-090`) aussi.
6. **Un champ de nombre** (`Input`) présente une telle valeur avec `type="number"` et **sans `inputmode`** : le clavier du téléphone a alors le signe moins (ceux de `inputmode="numeric"` et `"decimal"` n'en ont pas sur l'iPhone), et l'ordinateur garde les flèches. Ses bornes peuvent être négatives, `Input(value: temperature, min: -50, max: 50)` ; « -12 » et « −12 » (le signe copié d'une page suédoise) sont compris.
7. **Partout où passe l'état** : l'état écrit porte le signe (`temperature=-7`) ; une valeur gardée (`keep`) revient avec lui ; des données reçues (`{"temperature": -3}`) vont dans une valeur qui peut être négative, jamais dans une autre ; un formulaire envoie « -3 », et le serveur le vérifie à nouveau, avec les bornes du champ ; sans JavaScript, `holo serve` fabrique la même page.
8. **Ce qui attend un nombre positif refuse une valeur négative**, avec la raison : une glissière, une barre, une case, une place sur un plateau, un dessin, `limit:` et `offset:`, un module, un fichier exporté (`Transfer`), l'adresse (`address:`), un chronomètre, une valeur partagée, une quantité qui a un prix.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Qui peut passer sous zéro | A. toute valeur, comme en JavaScript ; B. une valeur qui part de sous zéro ; C. **une valeur que la page nomme, `negative: [balance]`** ; D. un plancher pour chaque valeur, `State(balance: Number(0, min: -500))` ; E. un signe devant le départ, `State(balance: +0)` | A remet le panier à −1 et change les pages qui existent ; B oublie le solde qui part de 0, et changer le départ changerait le plancher sans prévenir ; D est long et invente un bloc, alors que le champ a déjà `min:` ; E est obscur, et `-0` est un défaut de JavaScript. C tient en une ligne, se lit, et le moteur refuse un nom oublié |
| Le mot | `signed` (C, Rust) ; `belowZero` ; **`negative`** | `signed` se lit « signé » en français, comme une lettre ; `belowZero` est long ; `negative` se comprend sans être programmeur, et ne change le sens d'aucun mot du web |
| La division entière et l'arrondi | vers le bas (−7 ÷ 2 = −4, Python et `Math.floor`) ; **comme sans le signe** (−3, Rust, C, Java) ; la moitié vers le haut (`Math.round(-2.5)` = −2) ; **la moitié loin de zéro** | un nombre négatif se calcule comme sans son signe : rien de nouveau à apprendre, et les deux sens sont symétriques ; pour des nombres positifs, rien ne change (`ADR-043`, `ADR-066`) |
| Le signe moins | le trait d'union partout (`String(-5)`) ; le vrai signe « − » partout ; **celui de la langue (CLDR)** | le trait d'union seul est faux en suédois ; « − » partout ne se colle pas comme un nombre dans un tableur ; celui de la langue est ce que montre `Intl.NumberFormat`, et se copie bien en français et en anglais |
| Le clavier du champ | `inputmode="numeric"` ; `type="text"` ; **`type="number"` sans `inputmode`** | `numeric` n'a pas de signe moins sur l'iPhone ; un texte perd les flèches et la vérification du navigateur ; `type="number"` seul donne un clavier qui a le signe moins sur Android comme sur l'iPhone |
| Ce que le moteur garde | changer toutes les valeurs en nombres signés ; **garder les mêmes nombres, en complément à deux pour un négatif** | les valeurs positives ne dépassent jamais 10¹⁵ : relues signées, elles ne changent pas ; aucun fichier partagé n'est réécrit, seul l'arbitre calcule avec le signe |

## Ce qui est refusé, et pourquoi

- `State(t: -5)` sans `negative: [t]` : « « t » commence sous zéro : pour une valeur qui peut être négative, écris negative: [t] ».
- `negative:` qui n'est pas une liste de noms ; un nom inconnu, écrit deux fois, un texte, une liste, l'heure du visiteur, une valeur partagée, une quantité qui a un prix.
- `t.add(-5)`, `t.sub(-5)` : le bon mot est donné. `cart.set(-1)`, `If(cart, under: -1)`, `Input(value: cart, min: -5)` pour une valeur qui ne descend pas sous zéro.
- Une valeur négative dans une glissière, une barre, une case, un plateau, un dessin, `limit:`, un module, `Transfer`, `address:`, `{t:stopwatch}`.

## Les défauts du web évités

- **Le compteur qui passe à −1** : seules les valeurs nommées descendent sous zéro.
- **`-0`** : `Intl.NumberFormat("fr").format(-0)` écrit « -0 », et `(-0.004).toFixed(2)` « -0.00 » ; ici, un nombre exact n'a pas de zéro négatif.
- **`Math.round(-2.5)` qui vaut −2** quand `Math.round(2.5)` vaut 3 : ici, l'arrondi est le même des deux côtés de zéro.
- **Le signe moins d'une seule langue** (`String(-5)`), et le signe qui passe à droite du nombre dans une phrase en arabe : ici, celui de la langue de la page.
- **Le clavier du téléphone sans signe moins** (`inputmode="numeric"` sur l'iPhone) : ici, il l'a.

## Dettes

- Une glissière (`Slider(min: -10)`), une valeur partagée, l'adresse et un fichier exporté ne prennent pas encore de nombre négatif.
- Le plancher d'une valeur pour les règles reste −1 000 000 000 : le `min:` d'un champ ne borne que la saisie, comme pour une valeur positive.
- La lecture au TalkBack de « -7 °C » (« moins sept degrés ») est à essayer sur le téléphone de Yocthan.

## Critères de validation

- Tests du moteur (`negative.rs`) :
  - `a_value_goes_below_zero_only_when_the_page_says_so` : −2 − 5 = −7, `set(-10)`, `mul(-1)`, −7 ÷ 2 = −3, un écart, −12,50 − 0,25 = −12,75, × 1,1 = −14,03, le panier qui reste à 0, la borne d'un milliard, l'état écrit et relu, `keep` ;
  - `a_negative_number_is_compared_and_shown_in_the_page_language` : les conditions, la page en français et en suédois, le titre, les formats `d0`, `00`, `number`, `cents`, `nd2` ;
  - `a_field_takes_a_negative_number_with_a_keyboard_that_has_the_minus_sign` : « -12 », « −3 », les bornes du champ, ce qui n'est pas un nombre, le champ sans `inputmode` ;
  - `data_forms_and_the_server_keep_the_sign` : des données reçues, un formulaire et sa vérification par le serveur, un geste sans JavaScript ;
  - `what_cannot_be_negative_is_refused_with_the_reason` : vingt-deux refus, et ce qui est permis ;
  - `a_component_shows_and_changes_a_negative_value_it_is_given`.
- Dans Chrome : « des nombres négatifs : sous zéro, le signe moins de la langue, un champ dont le clavier l'a (leçon 125) ». Il rate si la page ne garde pas le signe (avant cette décision, « Temperatur: 0 °C » au lieu de « −7 »).
- Leçon `125-des-nombres-negatifs.holo`.
