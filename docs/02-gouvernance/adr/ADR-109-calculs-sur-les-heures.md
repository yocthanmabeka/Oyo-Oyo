# ADR-109 — Les calculs sur les heures : une heure, un moment, `now`, `Minutes`, `{left:duration}`

- Statut : ACCEPTÉ (Yocthan, 2026-10-09 : « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-10
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #238 (« Dernière dette du web : les calculs sur les heures »), dans la file des dernières dettes du web ouverte par Yocthan le 2026-10-09 ; le grand tableau du web, où `Date` était « en partie » (« pas encore d'heure seule qui se compare, ni de compte à rebours en heures et minutes », `docs/01-holocode/TABLEAU-WEB.md`) ; les dates (`ADR-067`) ; l'heure du visiteur (`ADR-039`) ; les secondes et le chronomètre (`ADR-089`) ; les formats après deux-points (`ADR-043`) ; les nombres négatifs (`ADR-102`) ; une date pour les machines (`ADR-098`) ; le champ heure (`ADR-042`) ; l'écriture des noms (`ADR-016`, `ADR-037`) ; la règle de parité de Yocthan (2026-10-07).
- Validation : faite par Yocthan le 2026-10-09 (« tu le valides déjà, tu le fais déjà »).
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Une page savait compter des jours entiers (`Days`, `ADR-067`), montrer l'heure du visiteur à la minute (`{hour} h {minute:00}`, `ADR-039`) et faire tourner un chronomètre (`ADR-089`). Elle ne savait pas compter les heures et les minutes entre deux moments, ajouter une durée à une heure, ni écrire « le train part dans 2 h 15 ». Un champ heure (`Input(type: time)`, `ADR-042`) donnait un texte que rien ne comprenait, et acceptait même « 25:99 ».
- Le web le fait avec `Date`. Ses défauts :
  - **deux fuseaux pour deux écritures voisines** : `new Date("2026-10-10")` est lu en temps universel, `new Date("2026-10-10T14:30")` à l'heure de l'appareil ; à Montréal, la première date s'affiche la veille ;
  - **un `Date` est toujours un instant** : « 18:45 » n'existe pas seul, il devient l'heure d'un jour et d'un fuseau qu'on n'a pas choisis ;
  - **la nuit du changement d'heure** : ajouter 24 h en millisecondes n'est pas ajouter un jour ; et une page qui compte elle-même à l'horloge (« 22:00 → 04:00 = 6 h ») se trompe d'une heure la nuit où l'heure revient ou saute ;
  - **le compte à rebours qui dérive** : un `setInterval` qui retire 1 chaque seconde prend du retard, car le navigateur ralentit les minuteries d'un onglet caché (une fois par minute après cinq minutes, dans Chrome) ; revenu dans l'onglet, le visiteur lit un compte faux ;
  - **« 2 h 15 » écrit à la main**, dans une seule langue ; `Intl.DurationFormat` n'est dans tous les navigateurs que depuis 2025, et écrit une chaîne vide pour une durée nulle (vu dans Chrome : « dans . ») ;
  - **le lecteur d'écran** : un compte posé dans une zone `aria-live` parle à chaque changement, sans arrêt.

## Décision

```holo
Page(
  title: "The train",
  state: State(train: "18:45", concert: "2026-12-31T20:30", arrival: "09:00", departure: "17:30", meeting: "14:00"),
  computed: [
    Minutes(name: left, from: now, to: train),
    Minutes(name: wait, from: now, to: concert),
    Minutes(name: worked, from: arrival, to: departure),
  ],
  children: [
    P("It is {now:time}."),
    If(now, under: train, children: [ P("The {train:time} train leaves in {left:duration}.") ]),
    P("The concert, on {concert:weekday} {concert:date} at {concert:time}, starts in {wait:duration}."),
    Input(value: arrival, label: "Arrival", type: time),
    Input(value: departure, label: "Departure", type: time),
    Button(name: ClockIn, text: "Clock in"),
    P("Worked: {worked:duration}."),
    Button(name: Later, text: "15 min later"),
    P("The meeting starts at {meeting:time}."),
  ],
  rules: [
    On(ClockIn.tap, effect: arrival.set(now)),
    On(Later.tap, effect: meeting.add(15min)),
  ],
)
```

1. **Une heure est un texte « HH:MM »**, de 00:00 à 23:59 : la forme que donnent le champ heure et les données JSON (« 18:45:30 » aussi : les secondes ne comptent pas). **Un moment est un texte « AAAA-MM-JJTHH:MM »**, une date et une heure (ISO 8601, la forme de `<input type="datetime-local">`). Une valeur est une heure quand elle est déclarée avec une heure, `State(train: "18:45")`, ou qu'un `Input(type: time)` la présente ; un moment, quand elle est déclarée avec un moment. **Sans fuseau** : c'est l'heure de l'horloge du visiteur, jamais convertie en silence.
2. **`now`** est le moment présent, « 2026-10-10T14:30 », donné par l'appareil du visiteur (ou par le serveur qui fabrique la page), comme `today`, `hour` et `minute`. La page le tient à jour à chaque minute. On ne le déclare pas, on ne le garde pas, on ne l'écrit pas.
3. **`Minutes(name: left, from: now, to: train)`**, dans `computed: [ … ]`, compte les minutes, comme `Days` compte les jours :
   - **entre deux moments, les vraies minutes**, d'après les changements d'heure du fuseau du visiteur : la nuit du 24 au 25 octobre 2026 à Paris, de 22 h à 4 h, 7 h et pas 6 ; du 24 octobre à 16 h 30 au 31 décembre à 20 h 30, 68 jours et 5 heures ;
   - **vers une heure seule, jusqu'à la prochaine fois que l'horloge la montre** : de 22:00 à 06:00, 8 h (le travail de nuit) ; de `now` à « 18:45 », jusqu'à 18 h 45 aujourd'hui, ou demain si elle est passée ; d'une heure seule à un moment, depuis la dernière fois que l'horloge l'a montrée ;
   - 0 si une valeur manque, ou si le moment `to` est passé ; sauf si la page le dit, `negative: [late]` (`ADR-102`) : le compte passe alors sous zéro ;
   - il se montre (`{left}`, en minutes), se compare (`If(left, under: 60)`), se guette (`When(left, is: 0, effect: …)` : le compte arrive à zéro), sert dans une demande (`pay.mul(worked)`, `end.add(left)`).
4. **`{left:duration}`** écrit une durée dans la langue de la page, comme `Intl.DurationFormat` (style « short », d'après le CLDR) : « 2 h et 15 min », « 2 hr, 15 min », « 2 Std., 15 Min. », « 3 j, 4 h et 5 min » ; les jours, puis les heures, puis les minutes ; ce qui vaut zéro ne s'écrit pas, mais une durée nulle s'écrit « 0 min ». Un compte négatif prend le signe moins de la langue. Le français, l'anglais, l'allemand, l'espagnol, l'italien, le portugais, le néerlandais et le suédois ont leurs mots ; les autres langues, ceux de l'anglais pour l'instant, comme les noms des jours (`ADR-043`).
5. **`{train:time}`** écrit une heure dans la langue de la page, comme `Intl.DateTimeFormat` (`timeStyle: "short"`) : « 18:45 » en français, « 6:45 PM » en anglais, « 9:05 » en espagnol. Un moment se montre aussi par sa date, `{concert:date}` et `{concert:weekday}` (`ADR-067`). **Pour les machines** (`ADR-098`), une heure, un moment et une durée montrés sont des `<time>` : `datetime="18:45"`, `datetime="2026-12-31T20:30"`, `datetime="PT2H15M"`.
6. **Une heure avance ou recule d'une durée écrite avec son unité**, les unités `min` et `h` du langage : `meeting.add(15min)`, `meeting.sub(2h)`, `meeting.add(1.5h)` ; ou d'un nombre de minutes de la page, `end.add(length)`. Une heure seule fait le tour du cadran (23:50 + 15min = 00:05) ; un moment change de jour, en vraies minutes. **`arrival.set(now)`** : une heure prend l'heure présente (« 14:30 »), un moment le moment entier. Comme tout texte, seulement par un geste (`ADR-044`).
7. **Deux heures, deux moments se comparent dans le temps** : `If(now, under: train)`, `If(now, over: "18:45")`, `When(now, is: "07:00", effect: …)` (un réveil). Un moment face à une heure seule se compare par son heure ; face à une date, par sa date. `is` et `not` comparent aussi à un texte vide, `If(arrival, is: "")`.
8. **Le compte suit l'horloge, il n'est jamais décompté.** La page redonne l'heure au moteur au début de chaque minute, et tout de suite quand l'onglet revient au premier plan (un navigateur ralentit les minuteries d'un onglet caché) ; le moteur refait chaque compte d'après elle. Il ne dérive donc pas, et il arrive à zéro à la minute dite.
9. **Le fuseau du visiteur** : au démarrage, la page cherche, d'après l'appareil (`getTimezoneOffset`), les changements d'heure de l'année passée et de l'année qui vient, à la minute près, et les donne au moteur avec la minute présente en temps universel. Le moteur n'embarque aucune base des fuseaux. Un moment qui tombe deux fois (l'heure répétée de l'automne) est pris la première fois ; un moment qui n'existe pas (l'heure sautée du printemps) avance d'autant, comme le fait `Date`.
10. **Le lecteur d'écran** n'entend pas un compte à chaque minute : il n'est dans aucune région que le lecteur annonce, et la page ne dit rien quand il change. On le lit en arrivant dessus, comme tout texte.
11. **Sans JavaScript** (`holo serve`), le serveur fabrique la page avec ses comptes ; une heure écrite dans un champ part avec le toucher, et le serveur la recompte (de 22:00 à 06:00 : « 8 h »). `now` y est l'heure du serveur, en temps universel, comme `hour` et `minute` (`ADR-039`) ; avec JavaScript, le moteur la corrige à son arrivée, car une page qui lit l'heure le fait venir tout de suite.
12. **Parité** : ce sont des textes, les mêmes au doigt, à la souris, au clavier et au lecteur d'écran ; les boutons qui décalent une heure sont des boutons ordinaires ; le champ heure est celui du navigateur, avec son clavier sur un téléphone.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| La forme d'une heure | A, **un texte « HH:MM »**, comme une date est un texte « AAAA-MM-JJ » (`ADR-067`) ; B, une valeur à part, `State(train: Time(18, 45))` ; C, un nombre de minutes depuis minuit, `State(train: 1125)` | A : c'est ce que donnent le champ heure et le JSON, rien à convertir, l'ordre des textes est celui du temps, et c'est le choix déjà fait pour les dates. B est un mot de plus et des conversions au champ et aux données ; C est illisible pour qui écrit la page |
| Le moment présent | **`now`**, un mot réservé ; un bloc `Clock` à déclarer ; `today` avec `hour` et `minute` | `now` : le mot de JavaScript (`Date.now()`), de Python, de SQL, de Temporal ; il se lit seul, et `start.set(now)` se comprend. Le prix : `now` ne sert plus de nom de valeur ; aucune leçon, aucun exemple ne s'en servait, et le moteur le dit avec le bon message |
| Compter | **`Minutes(name:, from:, to:)`** ; `Duration(…)` ; `Hours(…)` ; `Days(…, unit: minutes)` | `Minutes`, à côté de `Days` : le nom dit l'unité du nombre qu'on compare (`If(left, under: 60)`) et qu'on montre (`{left}`) ; `Duration` ne dit pas l'unité ; `Hours` mentirait, on compte des minutes ; une unité dans `Days` mêlerait deux comptes sous un seul mot |
| Le fuseau et le changement d'heure | compter à l'horloge, comme `Temporal.PlainDateTime` (un jour fait toujours 24 h) ; embarquer la base des fuseaux (IANA) ; **la page donne au moteur les changements d'heure du visiteur, d'après l'appareil** | les vraies minutes, sans base des fuseaux dans le moteur (des centaines de kilo-octets) : l'appareil connaît déjà le décalage de chaque minute. Compter à l'horloge ferait dire « 6 h » pour une nuit de 7 h, et un compte à rebours d'un mois se tromperait d'une heure pendant des jours |
| Une heure seule dans un compte | la même journée, négative si elle est passée ; **la prochaine fois que l'horloge la montre** | le travail de nuit (22:00 → 06:00 : 8 h) et « la boutique ouvre dans 13 h » sont justes ; un événement qui n'arrive qu'une fois s'écrit avec sa date, et son compte s'arrête à zéro (ou passe sous zéro, `negative:`) |
| Décaler | `add(15)`, l'unité devinée d'après la valeur ; **`add(15min)`, `add(2h)`, avec les unités du langage** | `15` ne dit pas s'il s'agit de minutes ou d'heures ; `min` et `h` existaient déjà dans le langage, sans emploi, et `3s` s'écrit ainsi depuis `ADR-039`. `add(15)` est refusé avec le bon mot |
| Écrire une durée | « 2 h 15 », à la française ; **le style « short » du CLDR, celui d'`Intl.DurationFormat`** : « 2 h et 15 min », « 2 hr, 15 min » ; le style « narrow », « 2h 15min » | le style « short » est la norme du web : l'essai le compare à ce qu'écrit le navigateur lui-même. « 2 h 15 » n'a pas d'équivalent dans les autres langues ; « narrow » se lit mal à voix haute. Et « 0 min » plutôt que rien |
| Les jours dans une durée | « 76 h et 5 min » ; **« 3 j, 4 h et 5 min »** | plus lisible dans un compte à rebours ; un jour y vaut 24 h de vraies minutes |
| Le lecteur d'écran | une zone `aria-live` ; `role="timer"` ; **un texte ordinaire, jamais annoncé** | un minuteur ne s'annonce pas (`role="timer"` lui-même ne parle pas) ; le chronomètre (`ADR-089`) annonce son temps final quand on l'arrête, un compte à rebours n'en a pas. Une règle peut faire quelque chose quand il arrive à zéro, `When(left, is: 0, …)` |
| Rafraîchir | chaque seconde ; **au début de chaque minute, et quand l'onglet revient** | le compte est à la minute ; une page qui ne montre pas la seconde ne réveille pas l'appareil chaque seconde (la batterie, `ADR-089`) |

## Ce qui est refusé, et pourquoi

- `State(now: …)`, `keep: [now]`, `address: [now]`, `visit: [now]`, `negative: [now]`, `Input(value: now)`, `Module(output: now)` : le moment présent est donné par le moteur ; le garder, l'écrire ou le faire voyager le rendrait faux.
- `Minutes` sans nom, avec un nom en majuscules ou pris par une autre valeur, sans `from:` ou `to:`, avec un réglage inconnu ; `Minutes(from: today, …)` (une date : les jours se comptent par `Days`) ; une valeur qui n'est ni une heure ni un moment.
- `meeting.add(15)` (sans unité), `meeting.add(30s)` (une heure se compte à la minute), `meeting.add(1.01h)` (pas une minute entière), `meeting.add(price)` (un nombre à virgule), une durée de plus d'une année ; `meeting.set("25:00")` ; `meeting.mul(2)`.
- `{train:duration}` (une heure n'est pas un nombre de minutes : `{train:time}`), `{length:time}` (un nombre n'est pas une heure), `{train:date}` (une heure seule n'a pas de date), `{concert:number}`, `{price:duration}` (un nombre à virgule).
- `If(train, over: "2026-12-24")` (une heure seule face à une date), `If(train, over: buyer)` (une heure face à un texte), avec la raison.
- Dans un champ heure, une heure qui n'existe pas à l'horloge (« 25:99 », « 24:00 ») : la valeur ne change pas.

## Les défauts du web évités

- **Les deux fuseaux de `Date`** : ici, une heure et un moment n'ont pas de fuseau caché ; c'est l'heure de l'horloge du visiteur, toujours.
- **La nuit du changement d'heure** : le compte entre deux moments est celui des vraies minutes ; ajouter 2 h à 1 h 30 la nuit du retour à l'heure d'hiver donne 2 h 30 à l'horloge, comme la montre au poignet.
- **Le compte à rebours qui dérive** : refait d'après l'horloge à chaque minute et au retour dans l'onglet, jamais décompté.
- **« 2 h 15 » écrit à la main dans une langue** : le moteur écrit la durée dans la langue de la page, comme `Intl.DurationFormat`, et « 0 min » là où il n'écrit rien.
- **« 25:99 » accepté** par un champ heure : refusé.
- **Le compte qui parle sans arrêt** au lecteur d'écran : il ne s'annonce pas.

## Dettes

- Un moment avec son fuseau, celui des données (« 2026-12-31T19:30Z », « …+01:00 ») : un direct à heure fixe pour le monde entier, montré à l'heure de chaque visiteur. Aujourd'hui, un tel texte n'est pas un moment.
- Un champ pour un moment, date et heure ensemble (`datetime-local`) ; les bornes d'un champ heure (`Input(type: time, min: "09:00")`).
- Une heure dans les éléments d'une liste (`{item.start:time}`).
- Les secondes dans un compte (le décollage d'une fusée) : le compte est à la minute.
- Les variantes régionales suivent leur langue de base, comme les dates : « 09 h 05 » au Canada, « 2 hrs, 15 mins » au Royaume-Uni, « 9:05 a.m. » au Mexique ne sont pas encore écrits ainsi.
- Deux heures sans date comptent à l'horloge : la nuit du changement d'heure, 22:00 → 06:00 fait 8 h ; avec les dates (deux moments), le compte est juste. Les changements d'heure connus sont ceux de l'année passée et de l'année qui vient.
- Sans JavaScript, `now` est l'heure du serveur, en temps universel.
- L'éditeur ne propose pas encore `now`, `time` ni `duration` en écrivant.
- Le grand tableau du web : la ligne `Date` peut passer à « oui », et « une heure seule n'est pas encore un time » (`ADR-098`) n'est plus vrai, à la prochaine publication.
- La lecture au TalkBack de « 2 h et 15 min » est à essayer sur le téléphone de Yocthan.

## Critères de validation

- Tests du moteur (`moteur/src/hours.rs`) :
  - `hours_and_moments_are_read_and_written` : lire et écrire une heure et un moment, dix textes refusés, décaler (le tour du cadran, le changement de jour), les durées écrites, les comparaisons ;
  - `hours_and_durations_are_written_like_intl` : les heures et les durées de huit langues, comparées à ce qu'écrivent `Intl.DateTimeFormat` et `Intl.DurationFormat` dans Chrome ; « 0 min » ; le signe moins ; la forme pour les machines ;
  - `the_visitors_zone_counts_the_real_minutes` : Paris en 2026, 7 h de 22 h à 4 h la nuit du 25 octobre, 23 h de midi à midi le 29 mars, l'heure répétée et l'heure sautée, `now` pendant l'heure répétée, et sans fuseau ;
  - `minutes_count_between_two_times_and_follow_the_clock` : les comptes de la page, les gestes, le champ heure (la nuit, « 25:99 » refusé), la minute qui passe, `When(left, is: 0)` et `When(now, is: "07:00")`, le titre de l'onglet ;
  - `the_page_shows_hours_and_durations_for_people_and_machines` : la page fabriquée, ses `<time datetime>`, en français et en anglais ;
  - `a_form_counts_hours_without_javascript` : les champs et un toucher envoyés à `holo serve` ;
  - `what_is_not_an_hour_is_refused_with_the_reason` : vingt-cinq refus, et ce qui est permis.
- Dans Chrome : « des heures : un compte à rebours qui suit l'horloge, les vraies minutes la nuit du changement d'heure, la langue de la page, au clavier et sans JavaScript (leçon 132) ». Une horloge tenue par l'essai, à Paris, le 24 octobre 2026 à 16 h 30 : « Il est 16:30. », le train dans « 2 h et 15 min », le concert dans « 68 j et 5 h », chaque texte comparé à ce qu'écrit `Intl` ; l'onglet qui revient à 18 h 44, 18 h 45, 18 h 46 ; rien d'annoncé ; la nuit de 22:00 à 06:00, 8 h ; pointer, au doigt ou à la souris ; la réunion au clavier ; sans JavaScript avec `holo serve` ; à 360 de large ; l'audit axe-core. Il rate sans les changements d'heure du visiteur (« 68 j et 4 h »), et sans le retour de l'onglet (le compte reste à 16 h 30).
- Leçon `132-des-heures.holo`.
