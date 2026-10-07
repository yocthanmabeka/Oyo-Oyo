# Piste 10 — Calculs purs et extensions encadrées

> Statut : EXPLORATION. Avis de Claude, pas une décision.
>
> **Réparé le 2026-10-07, après cette exploration** : la boîte des modules (PR 141) ; la leçon 69 rend de nouveau 5 050. Vérifié dans Chrome sans fenêtre, avant et après.

## Ce que Codex demandait

Sa ligne, citée telle quelle (issue #82, texte de Codex transmis par Yocthan, écrit sur la base `a142a4c`) :

> | **10** | **Calculs purs et extensions encadrées** : valeurs calculées, dates, montants exacts, modules spécialisés. | Couvrir les comportements particuliers qu’une collection de blocs prédéfinis ne peut pas tous prévoir. |

« Calcul pur » veut dire : un calcul qui lit des valeurs et rend un résultat, sans rien changer d'autre. « Extension encadrée » : du code venu d'ailleurs, enfermé dans une boîte que le moteur contrôle.

## État vérifié (main, 7a48def, 2026-10-07)

> Mise à jour de la relecture : pendant la relecture, `main` est passé de `1119361` à `9df3b2b` (PR 140 à 142). La PR 141 (`e46fd4e`, fusionnée le 2026-10-07 à 15 h 15) répare la boîte des modules (voir l'alerte). Les numéros de ligne de ce rapport valent pour `7a48def` et `1119361` ; sur `9df3b2b`, ceux de `page-engine.js` au-delà de la ligne 869 sont décalés de 2 à 4 (la boîte : `:1177-1188` ; la page : `:1201-1223`).

**En une phrase : les briques sont là, mais rien ne les relie.** HoloCode calcule en nombres entiers exacts, écrit les centimes et les noms des jours, et sait enfermer un module. Mais une valeur ne se calcule jamais seule d'après une autre (sauf `count`, `total` et l'heure), on ne compte pas en jours, un pourcentage tombe toujours vers le bas, et un module n'échange qu'un nombre contre un nombre.

**Alerte (réparée pendant la relecture) : du 2026-10-07 à 10 h 29 (PR 136) au même jour à 15 h 15 (PR 141), aucun module ne marchait dans le navigateur.** Le passage du code en anglais (PR 136, commit `0fde532`, ADR-060) a traduit la page, mais pas le petit programme de la boîte, écrit dans une chaîne de texte :

- la boîte attendait (jusqu'à `1119361`) `{ octets, entree, pages }` et répondait `{ debut }`, `{ ok, sortie }`, `{ ok, raison }` (`moteur/web/page-engine.js:1173-1184`) ;
- la page envoie `{ bytes, entry, pages }` et lit `data.start` et `result.output` ; ses propres échecs portent `reason`, quand la boîte écrit `raison` (`page-engine.js:1197-1219`).

Le module ne reçoit donc pas ses octets, échoue tout de suite, et la page émet `Nom.failed`. Dans la leçon 69 : « Calculer la somme » n'affiche plus rien ; « Lancer la boucle sans fin » affiche aussitôt « La boucle a été arrêtée après deux secondes », et « Demander 64 Mo » affiche « Refusé » : **deux faux succès**. (Déduit du code et de la reproduction dans Node ; je ne l'ai pas vu dans un navigateur.) Avant la traduction (`40b9403`), les clés concordaient (`page-moteur.js:1129-1174`). Les critères de validation d'ADR-060 ne citaient pas la leçon 69. Reproduit dans Node avec le vrai code de la boîte et le vrai module (voir « Ce que j'ai exécuté »). La réparation tient en quelques lignes.

**Réparé par la PR 141** (`e46fd4e`, fusion `55abd1e`), pendant la relecture : la boîte attend maintenant `{ bytes, entry, pages }` et répond `start`, `output`, `reason`. Vérifié à la relecture : `node essais/sonde-module.mjs 9df3b2b` rend `[{"start":true},{"ok":true,"output":5050}]`. Le journal (`docs/06-journal/JOURNAL.md`, entrée du 2026-10-07) dit l'épreuve faite dans Chrome sans fenêtre : la leçon 69 rend 5 050. Reste à faire : un essai automatique qui joue la boîte comme la page ; la PR 141 n'en ajoute pas, et le journal propose un essai dans un navigateur pour chaque PR qui touche `moteur/web/`.

**Ce qui existe** :

| Capacité | Où | Décision, leçon |
|---|---|---|
| Des valeurs entières de 0 à 1 000 000 000, des textes, des listes ; 32 valeurs au plus | `moteur/src/state.rs:20-21`, `:606`, `:610`, `:1079` | ADR-023, ADR-027, ADR-044 |
| Les demandes `add`, `sub`, `set`, `random`, `mul`, `div` ; une autre valeur peut servir de quantité (`best.set(score)`) | `state.rs:514`, `:1102-1129`, `:1514-1537` | ADR-023, ADR-032, ADR-043 ; leçons 13, 29, 66 |
| La division entière, arrondie vers le bas ; diviser par 0 écrit est refusé, par une valeur nulle ne change rien | `state.rs:1122`, `:1525-1526` | ADR-043 |
| Deux valeurs calculées par le moteur : `count` et `total`, d'après `Prices` ; et le nombre d'éléments d'une liste | `state.rs:886-937` ; `lists.rs:497-500` | ADR-023 ; leçon 14 |
| L'heure du visiteur (`year` … `minute`), tenue à jour chaque minute ; donnée par le serveur avec `HOLO_NOW`. Elle sert déjà de quantité dans une demande : le moteur la range dans l'état, donc `best.set(hour)` passe (`state.rs:949-955`, `rules.rs:125`, `:278` ; `relecture-1-10/year-qty.holo` et `.test` : « ok », `best` vaut 2026 après `best.set(year)`) | `state.rs:962-996` ; `page-engine.js:337-354` ; `src/bin/holo.rs:120-129` | ADR-039 ; leçon 48 |
| Les formats `{n:00}`, `{n:number}`, `{n:cents}`, `{weekday:name}`, `{month:name}`, selon la langue de la page | `moteur/src/format.rs:20-69` | ADR-043 ; leçon 67 |
| Un champ de date : `Input(type: date)` ; sa valeur est un texte (`"2026-10-10"`) | `state.rs:1236-1241` | ADR-042 ; leçon 60 |
| Les modules enfermés : un fil à part, une mémoire plafonnée (64 KB à 16 MB), un temps borné (10 ms à 5 s), huit au plus, un nombre en entrée, un en sortie | `moteur/src/modules.rs:23-30`, `:87-123`, `:126-134` ; `page-engine.js:1169-1219` | ADR-045 (ADR-011, partie C) ; leçon 69 |
| Le principe : les « appels de calcul » purs sont permis (« `distance(p, d)`, `min(a, b)`, le total d'un panier ») ; le JavaScript libre est remplacé par « des règles plus riches, des fonctions pures, des modules enfermés » | `docs/02-gouvernance/adr/ADR-015-regle-des-appels.md:20` ; `ADR-035-…md:24` | ADR-015, ADR-035 |

**Ce qui manque** (vérifié) :

1. **Une valeur qui se tient à jour seule.** Hors `count`, `total`, l'heure et le nombre d'éléments d'une liste, tout calcul demande un geste et une suite de demandes (la leçon 66 a un bouton « Calculer le prix », `66-calculer.holo:21`). `When` ne se déclenche qu'au moment où sa condition devient vraie (`state.rs:1481-1510`) : `When(qty, over: 0, effect: [line.set(1999), line.mul(qty)])` calcule à 0 → 3, mais plus à 3 → 4 (essai `when.test` : « « line » vaut 5997, et l'essai attendait 7996 »).
2. **Calculer à partir de `total`.** `vat.set(total)` est refusé : une demande ne lit que les valeurs rangées dans l'état, et le moteur n'y range ni `count` ni `total` (`state.rs:1116-1119`, `:916-937` ; `n.set(count)` est refusé de même, `relecture-1-10/count-qty.holo`). L'heure, elle, y est rangée et passe (ligne « L'heure du visiteur » ci-dessus). Le message conseille « state: State(total: 0) », ce que le moteur refuse ensuite avec `prices:` (`state.rs:943-947`). Avec l'écriture d'aujourd'hui, la TVA d'un panier tenu par `Prices` est donc impossible, même par un bouton.
3. **La somme d'une liste.** `Prices` ne s'applique qu'à des valeurs déclarées une à une (`state.rs:900-902` ; essai T3 de la piste 1). Un panier reçu du serveur n'a pas de total. Et le prix s'écrit deux fois, dans `Prices` et dans `Item` (ADR-040, « Conséquences »).
4. **Un pourcentage exact.** `div` arrondit vers le bas : 20 % de 19,99 € donnent 3,99 € au lieu de 4,00 € (mesure : `expect vat = 399`) ; 20 % de 129,99 € donnent 25,99 € au lieu de 26,00 € (mesure : `expect vat = 2599`, total 155,98 € au lieu de 155,99 €).
5. **Un nombre négatif.** Impossible : `sub` s'arrête à 0 (`state.rs:1521`) ; 5 € moins 8 € donnent 0 (mesure). `State(rate: 0.2)` et `State(t: -3)` sont refusés (essais T6, T7). C'est voulu (ADR-023) ; je le note seulement.
6. **Les dates.** Il n'y a pas de valeur « date » : ni « dans trois jours », ni « J − 12 ». `{arrival:date}` est refusé (essai T5). Une date choisie par le visiteur est un texte, qui ne se compare qu'au vide. ADR-043 le disait : « Pas encore de calcul sur les dates ». Les noms des jours et des mois n'existent qu'en français et en anglais (`format.rs:27-30`) ; une page en allemand ou en espagnol reçoit les noms français (`format.rs:63` ; `relecture-1-10/de.holo` avec `lang: "de"` : « mercredi 7. octobre 2026 »).
7. **Les textes.** Ni longueur (« il reste 120 caractères »), ni découpe : `docs/01-holocode/TABLEAU-WEB.md:624` le classe à 20 %, « Plus tard ».
8. **Des modules plus riches.** Un nombre contre un nombre (`modules.rs:14`, `:95-97`) ; ADR-045 range « plusieurs nombres, un texte, une liste » dans « Restent à faire ». Et la boîte a été cassée de la PR 136 à la PR 141 (alerte ci-dessus).
9. **Petit écart** : `total` est borné au plus grand entier de la machine, pas à 1 000 000 000 comme les autres valeurs (`state.rs:927-929`).

**Documents en retard** :

- `docs/01-holocode/GUIDE.md:545` : « pas de centimes » pour le panier, alors que `{total:cents}` existe (ADR-043).
- `GUIDE.md:1364-1390` et la leçon 69 : justes sur le papier, mais la leçon ne marchait plus dans le navigateur de la PR 136 à la PR 141. ADR-045, « La preuve (dans Chrome, le 2026-10-06) », n'était plus vraie sur `main` entre ces deux PR ; elle l'est de nouveau sur `9df3b2b`, d'après le journal.
- `TABLEAU-WEB.md:623` donne 40 % aux modules ; dans le navigateur, c'était 0 % entre les PR 136 et 141.
- `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md:373` : « les listes ne contiennent que des textes » (faux depuis ADR-051) ; le reste de la phrase est juste.
- `ADR-060`, « Critères de validation » : la liste des leçons essayées dans Chrome n'a pas la 69.

**Ce que j'ai exécuté** (même moteur que pour la piste 1 : `holo.exe` reconstruit depuis `1119361` ; Node 22.21.0, PC Windows 11 ; fichiers dans `essais/`) :

```text
$ holo check essais/calcul.holo && holo test essais/calcul.holo essais/calcul.test
   ok
   calcul.test : ok, 13 ligne(s) jouée(s)
   (expect vat = 399 ; expect line = 0 après « type qty "3" » ; expect line = 5997 après « tap Line » ;
    expect line = 5997 après « type qty "4" » ; expect balance = 0 ; expect arrival = "2026-10-10")
$ holo check essais/p10/aujourdhui.holo && holo test essais/p10/aujourdhui.holo essais/p10/aujourdhui.test
   ok / aujourdhui.test : ok, 6 ligne(s) jouée(s)     (expect vat = 2599 ; expect ttc = 15598)
$ echo 'Page(… state: State(a: 1, vat: 0), prices: Prices(a: 1999), … On(V.tap, effect: vat.set(total)) ])' | holo check -
   ligne 1, colonne 149 : « vat.set(total) » : aucun nombre ne s'appelle « total » ; déclare-le sur la page, state: State(total: 0)
$ holo test essais/when.holo essais/when.test
   when.test, ligne 4 : « line » vaut 5997, et l'essai attendait 7996
T5 « {arrival:date} » : format inconnu ; formats possibles : 00 (zéros devant), number (1 234), cents (12,50), name (le nom du jour ou du mois)
T6, T7 une valeur se déclare par son nom et son départ, un nombre entier, un texte ou une liste : State(cart: 0, buyer: "", tasks: [])
$ node essais/sonde-module.mjs 7a48def        (le code de la boîte lu par git show, joué dans Node, avec exemples/lecons/69-compter.wasm)
   la page envoie : { bytes, entry: Number(entry), pages: Number(pages) }
   la boîte attend : { octets, entree, pages }
   message de la page (bytes, entry) → [{"ok":false,"raison":"WebAssembly.instantiate(): Argument 0 must be a buffer source or a WebAssembly.Module object"}]
     la page lit data.start : false ; result.output : undefined
   message avec octets, entree → [{"debut":true},{"ok":true,"sortie":5050}]
$ node essais/sonde-module.mjs 0fde532   → même échec (le commit de la traduction)
$ node essais/sonde-module.mjs 1119361   → même échec (main au moment de l'exploration)
$ node essais/sonde-module.mjs 9df3b2b   → [{"start":true},{"ok":true,"output":5050}]   (relecture : main après la PR 141)
$ git show 40b9403:moteur/web/page-moteur.js   (avant la traduction)
   1129: onmessage = async ({ data: { octets, entree, pages } }) => {   …   1166: boite.postMessage({ octets, entree: …, pages: … })
```

## Le scénario du site de référence

Le panier construit (`exemples/site-reference/panier.holo:21-22`) a trois articles à prix entiers en euros (`Prices(lever: 120, porte: 90, marche: 50)`), comme le demandait le cahier de Codex (« Trois articles à prix entiers : 120, 90 et 50 », `proposals/GPT5.6/site-reference-2026-10-06/README.md:37`). La recette F05 exige « aucun total calculé par une deuxième logique » quand on passe dans l'atelier (`RECETTE.md:19`). Un vrai panier va plus loin.

**La tâche** : le panier de « L'atelier des mondes », pour un client professionnel, montre des prix hors taxe en centimes, la TVA de 20 % au centime exact, le total toutes taxes comprises, le nombre de créations, et « Livraison prévue le samedi 10 octobre 2026 » (trois jours après aujourd'hui). Tout se tient à jour à chaque « Un de plus » ou « Un de moins », sans bouton « Calculer ». Le lecteur d'écran annonce le nouveau total. Le panier reste après un rechargement. Plus tard, les frais de port viendront d'un module du transporteur (poids, zone).

**Ce qu'il faut** : des valeurs qui se calculent seules ; la somme d'une liste (prix × quantité) ; un pourcentage arrondi au centime le plus proche ; une date, aujourd'hui plus trois jours, écrite en mots ; une annonce ; et, pour le transporteur, un module qui reçoit deux nombres.

## Options comparées

**A. Des valeurs qui se tiennent à jour**

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **A1. Des valeurs calculées, écrites avec des blocs** (recommandée) | `computed: Computed(subtotal: Sum(cart, of: item.price, times: item.qty), vat: Percent(20, of: subtotal))` | `computed` de Vue, un getter, une fonction appelée à chaque rendu | un getter ; un `Provider` dérivé (Riverpod) | Une seule vérité, recalculée par le moteur ; lisible partout (texte, `If`, `When`, quantité d'une demande) ; le serveur refait le même calcul | Une notion nouvelle et des mots ; il faut refuser les boucles (`a` qui dépend de `b` qui dépend de `a`) |
| A2. Des expressions avec des signes | `vat: subtotal * 20 / 100` | la même chose | la même chose | Court ; connu de tous | Des signes et leur ordre (`*` avant `+`), alors que HoloCode a choisi des mots (`is`, `over`) ; un arrondi caché ; ADR-043 a écarté les expressions |
| A3. Une règle qui suit chaque changement | `When(qty, changes, effect: [line.set(price), line.mul(qty)])` | `input.addEventListener("input", …)` | `onChanged:` | Un seul mot nouveau ; réutilise les demandes | L'auteur doit penser à chaque dépendance ; un oubli fait mentir l'écran, le défaut du web qu'ADR-023 voulait éviter |
| A4. Le serveur calcule et envoie | `Data` reçoit `subtotal`, `vat` | `fetch` | `http.get` | Les règles commerciales restent au serveur | Un aller-retour par geste ; rien sans réseau ; il faut un serveur qui calcule |

**B. Des montants exacts**

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **B1. Garder les centimes entiers ; un pourcentage arrondi au plus proche** (recommandée) | `Percent(20, of: subtotal)` : 0,5 centime monte | `Math.round(subtotal * 20 / 100)` sur des centimes | `(subtotal * 20 / 100).round()` | Exact, sans erreur de virgule ; une règle simple et écrite | Une seule règle d'arrondi pour tous ; les règles fiscales varient selon les pays (par ligne ou sur le total) |
| B2. Des nombres à virgule | `State(price: 19.99)` | `0.1 + 0.2 = 0.30000000000000004` | le paquet `decimal` | Écriture naturelle | Les erreurs d'arrondi du web, ou un type décimal à construire ; écarté par ADR-043 et ADR-051 |
| B3. Un réglage d'arrondi sur `div` | `vat.div(100, round: nearest)` | — | — | Presque rien à ajouter | Il faut toujours un geste pour calculer (manque 1) |

**C. Des dates**

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **C1. Une date est un nombre de jours, que le moteur sait lire et écrire** (recommandée) | `delivery: AddDays(today, 3)`, `{delivery:date}`, `wait: DaysUntil(opening)` | `new Date()`, `setDate(getDate() + 3)`, `toLocaleDateString("fr-FR", …)` | `DateTime.now().add(Duration(days: 3))`, `DateFormat` (paquet `intl`) | Se compare, se calcule, se rejoue ; le texte d'un `Input(type: date)` devient utilisable | Une date passée donne 0 jour (pas de nombre négatif) ; pas de fuseau horaire : l'heure est celle de l'appareil |
| C2. Seulement un format pour un texte de date | `{arrival:date}` sur `"2026-10-10"` | `toLocaleDateString` | `DateFormat` | Petit | Aucun calcul |
| C3. Le serveur envoie des textes prêts | `"Livraison le samedi 10 octobre"` dans le JSON | — | — | Rien à construire | La page ne compte pas les jours ; l'heure est celle du serveur |

**D. Des modules spécialisés**

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **D0. Réparer la boîte** (obligatoire, sans débat ; fait par la PR 141 pendant la relecture, sauf l'essai automatique) | rien ne change pour l'auteur | — | — | La leçon 69 remarche ; plus de faux succès | — |
| **D1. Plusieurs nombres nommés** (recommandée ensuite) | `Module(name: Shipping, source: "port.wasm", input: [weight, zone], output: [cost, days])` | `worker.postMessage({ weight, zone })` | `Isolate.run(() => …)` | Couvre les frais de port, une taxe, un score, une physique simple | Une convention d'échange (où le module lit et écrit ses nombres) à fixer ; la réponse arrive plus tard (`done`) |
| D2. Des textes et des listes | `input: articles`, `output: ranked` | un JSON dans le message | idem | Trier par pertinence, lire un format | Une surface plus grande ; un module plus lourd (estimation : 20 à 50 KB de plus pour lire du JSON) ; une copie à chaque appel |
| D3. Un module qui sert de valeur calculée | `computed: Computed(cost: Shipping(weight, zone))` | — | — | Direct | Un module tourne à part, sa réponse arrive plus tard : une valeur calculée doit être immédiate et rejouable |

**E. Les textes** : E1, `Length(message)` dans une valeur calculée (« il reste {left} caractères ») ; E2, rien, puisque `Input(max:)` borne déjà la longueur et que `text-transform` écrit les majuscules (leçon 50). Petit ; à faire quand un exemple le demandera.

**Noms (ADR-016)** :

| Nom proposé | Sur le web | Dans Flutter et Dart | Risque |
|---|---|---|---|
| `Computed`, `computed:` | Vue `computed`, Angular `computed()`, MobX : une valeur tirée d'autres, recalculée seule | un getter ; Riverpod | faible ; le moteur range déjà `count`, `total` et l'heure sous « computed » dans `holo vocabulary` |
| `Sum`, `of:`, `times:` | Excel `SOMME`/`SUM`, SQL `SUM`, Python `sum` ; « times » : « fois » | `fold`, `.sum` (paquet `collection`) | faible ; `times(2)` veut dire « deux appels » dans certains outils d'essai |
| `Percent(20, of: x)` | aucun mot en JavaScript ; `%` dans Excel | aucun | moyen si l'on écrit `Percent(x, 20)` : on ne sait plus qui est le taux. Le taux d'abord et `of:` nommé enlèvent le doute |
| `today` | Excel `AUJOURDHUI`/`TODAY()`, Python `date.today()`, SQL `CURRENT_DATE` | `DateTime.now()` | faible ; ne pas confondre avec `day` (le jour du mois) |
| `AddDays` | C# `AddDays`, date-fns `addDays` | `add(Duration(days: …))` | faible |
| `DaysUntil` | date-fns `differenceInDays` ; Excel `JOURS`/`DAYS(fin, début)` met la fin d'abord | `difference(…).inDays` | faible ; `DaysBetween(a, b)` aurait le piège de l'ordre |
| `{x:date}` | `toLocaleDateString` | `DateFormat` | faible : `date` existe déjà dans `Input(type: date)`, avec le même sens |
| `Status` | ARIA `role="status"` | `Semantics(liveRegion: true)` | faible ; « status » est aussi un code de réponse en HTTP |
| `input: [a, b]`, `output: [c, d]` | — | — | faible : les mêmes mots qu'aujourd'hui, au pluriel par la liste |

## Recommandation

Mon avis, dans cet ordre :

0. **Réparer la boîte des modules en premier** (c'est une réparation, pas un choix de langage ; elle passe quand même par une pull request) : remettre les mêmes noms des deux côtés du message (`page-engine.js:1173-1184` contre `:1197-1219`), ajouter aux tests un essai qui joue la boîte comme la page (ce que fait `essais/sonde-module.mjs`), puis refaire l'épreuve de la leçon 69 dans Chrome. Le dire à Yocthan : la leçon 69 affichait de faux succès depuis la PR 136. Mise à jour de la relecture : les noms et l'épreuve dans Chrome sont faits par la PR 141 (journal du 2026-10-07) ; reste l'essai automatique.
1. **Laisser `count` et `total` servir de quantité** (`vat.set(total)`), comme l'heure le fait déjà (`best.set(hour)` passe), et corriger le message trompeur. Aucun mot nouveau.
2. **Une seule notion nouvelle : les valeurs calculées (option A1)**, la même que la liste calculée de la piste 1. Une valeur calculée se déclare une fois, se lit partout, ne se demande jamais, et le moteur la recalcule après chaque changement, comme `count` et `total` aujourd'hui. Premières fonctions : `Sum` (de valeurs, ou d'une liste avec `of:` et `times:`), `Percent(… of:)` arrondi au centime le plus proche (option B1), `AddDays`, `DaysUntil`, `today` et le format `{x:date}` (option C1). Garder les entiers, les centimes et le zéro comme plancher.
3. **Modules** : après la réparation, plusieurs nombres nommés (option D1), quand un vrai module le demande (les frais de port du site de référence). Les textes et les listes ensuite, sur un exemple réel.
4. **Ne pas** ajouter de signes de calcul (`*`, `+`) ni de nombres à virgule.

Cela revient en partie sur deux refus : ADR-023 avait écarté `sum(price)` (« un pas vers les formules ») et ADR-043 les expressions. Je garde le second refus (pas de signes), je propose de lever le premier, parce que la somme d'une liste est le seul moyen d'avoir le total d'un panier reçu du serveur. À Yocthan de trancher.

## Exemple d'auteur

Le panier du scénario. Les lignes marquées `// proposé` n'existent pas. Fichier `essais/p10/propose.holo` (1 512 octets, 28 lignes utiles) ; `holo check` le refuse à son premier mot nouveau (« « Page » n'a pas de paramètre « computed » »). Sans ses lignes proposées, le reste passe aujourd'hui (`essais/p10/propose-sans-nouveau.holo` : « ok » ; `signal More.tap@1` est joué, et la liste garde ses 2 lignes ; la quantité changée, elle, ne se vérifie pas avec `holo test`, voir le défaut D7 de la piste 1).

```holo
// Le panier qui calcule seul : écriture PROPOSÉE (EXPLORATION, rien n'est décidé).
// Chaque ligne marquée « proposé » n'existe pas dans le moteur (main, 2026-10-07).
Page(
  title: "Panier",
  lang: "fr",
  state: State(cart: [
    Item(id: "barque", title: "La barque", price: 1999, qty: 1),
    Item(id: "phare", title: "Le phare", price: 11000, qty: 1),
  ]),
  keep: [cart],
  computed: Computed(                                     // proposé
    units: Sum(cart, of: item.qty),                       // proposé
    subtotal: Sum(cart, of: item.price, times: item.qty), // proposé : hors taxe, en centimes
    vat: Percent(20, of: subtotal),                       // proposé : au centime le plus proche
    total: Sum(subtotal, vat),                            // proposé
    delivery: AddDays(today, 3),                          // proposé : une date
  ),
  children: [
    H1("Votre panier"),
    Repeat(over: cart, children: [
      Row(gap: 8px, children: [
        Text("{item.title}, {item.price:cents} € HT : {item.qty}"),
        If(item.qty, over: 0, children: [ Button(name: Less, text: "Un de moins") ]),
        Button(name: More, text: "Un de plus"),
      ]),
    ], rules: [ On(Less.tap, effect: item.qty.sub(1)), On(More.tap, effect: item.qty.add(1)) ]),
    Status("{units} créations : {subtotal:cents} € HT, TVA {vat:cents} €, total {total:cents} € TTC."), // proposé
    P("Livraison prévue le {delivery:date}."),            // proposé : le format date
  ],
)
```

Au départ : « 2 créations : 129,99 € HT, TVA 26,00 €, total 155,99 € TTC. » Avec l'écriture d'aujourd'hui, la même page (`essais/p10/aujourdhui.holo`, 1 407 octets, 29 lignes utiles) doit écrire ses deux articles à la main, tenir elle-même un total hors taxe à chaque bouton, calculer la TVA par un bouton « Calculer la TVA », et trouve 25,99 € et 155,98 € ; elle ne sait pas écrire la date de livraison. Le bouton « Calculer » n'est pas obligatoire : chaque règle peut refaire la TVA à la suite de son geste (cinq demandes de plus par règle, et un oubli fait mentir l'écran ; `relecture-1-10/sans-bouton.holo` et `.test` : « ok », 169,97 € HT et TVA 33,99 € après deux « Un de plus »). Cela ne marche qu'avec des articles écrits à la main : dans les lignes d'une liste, `ht.add(item.price)` est refusé (T1 de la piste 1).

**Le même en HTML et JavaScript**, qui fait la même chose : centimes entiers, même arrondi (0,5 monte), mêmes bornes (0 à 1 000 000 000), date de l'appareil plus trois jours écrite en mots, mise à jour chaque minute (comme une page HoloCode qui lit l'heure ; à un écart près : la page HoloCode se cale sur le changement de minute, `page-engine.js:353`, la version web sur l'heure d'ouverture, donc jusqu'à près d'une minute de retard après minuit), panier gardé dans le navigateur, mêmes lignes gardées ou refaites, même annonce par `role="status"`, seulement quand le texte change. Fichiers `essais/p10/web/` : 4 068 octets, 81 lignes utiles (22 + 59), contre 1 512 octets et 28 lignes pour la page HoloCode (mesure, `wc -c` ; les octets comptent les commentaires des deux côtés, dont les marques « // proposé » ; sans aucun commentaire : 1 016 octets contre 3 626, mesure à la relecture avec `sed` puis `wc -c`).

```html
<!doctype html>
<html lang="fr">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Panier</title>
  <style>
    main { max-width: 40rem; margin: 0 auto; padding: 1rem; font-family: system-ui, sans-serif; }
    .row { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; }
    button:focus-visible { outline: 3px solid; outline-offset: 2px; }
  </style>
  <script type="module" src="panier.js"></script>
</head>
<body>
  <main>
    <h1>Votre panier</h1>
    <div id="lines"></div>
    <p id="status" role="status"></p>
    <p id="delivery"></p>
  </main>
</body>
</html>
```

```js
// panier.js — la même page que propose.holo, écrite en JavaScript.
const MAX = 1e9; // les bornes de l'arbitre : 0 à 1 000 000 000
const clamp = (n) => Math.max(0, Math.min(MAX, n));
export function totals(cart) {
  const units = clamp(cart.reduce((n, l) => n + l.qty, 0));
  const subtotal = clamp(cart.reduce((n, l) => n + l.price * l.qty, 0));
  const vat = clamp(Math.round((subtotal * 20) / 100)); // au centime le plus proche, 0,5 vers le haut
  return { units, subtotal, vat, total: clamp(subtotal + vat) };
}
export function addDays(now, days) {
  const d = new Date(now.getFullYear(), now.getMonth(), now.getDate()); // l'heure de l'appareil, à minuit
  d.setDate(d.getDate() + days);
  return d;
}
const money = new Intl.NumberFormat("fr-FR", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
export const cents = (n) => money.format(n / 100);
export const longDate = (d) => d.toLocaleDateString("fr-FR", { weekday: "long", day: "numeric", month: "long", year: "numeric" });

if (typeof document !== "undefined") {
  const start = [
    { id: "barque", title: "La barque", price: 1999, qty: 1 },
    { id: "phare", title: "Le phare", price: 11000, qty: 1 },
  ];
  let cart = start;
  try {
    const kept = JSON.parse(localStorage.getItem("cart"));
    if (Array.isArray(kept)) cart = kept.slice(0, 100).map((l) => ({ id: String(l.id ?? ""), title: String(l.title ?? "").slice(0, 200), price: clamp(Number.isInteger(l.price) ? l.price : 0), qty: clamp(Number.isInteger(l.qty) ? l.qty : 0) }));
  } catch {}
  const lines = document.getElementById("lines");
  function line(l, rank) {
    const row = document.createElement("div");
    row.className = "row";
    row.append(Object.assign(document.createElement("span"), { textContent: `${l.title}, ${cents(l.price)} € HT : ${l.qty}` }));
    const button = (text, change) => {
      const b = Object.assign(document.createElement("button"), { type: "button", textContent: text });
      b.addEventListener("click", () => { cart = cart.map((x, i) => (i === rank ? { ...x, qty: clamp(x.qty + change) } : x)); render(); });
      return b;
    };
    if (l.qty > 0) row.append(button("Un de moins", -1));
    row.append(button("Un de plus", +1));
    return row;
  }
  const built = new Map();
  function render() {
    // Une ligne qui n'a pas changé garde son nœud (et le focus) ; une ligne changée est refaite.
    const nodes = cart.map((l, rank) => {
      const key = JSON.stringify(l) + "#" + rank;
      if (!built.has(key)) built.set(key, line(l, rank));
      return built.get(key);
    });
    nodes.forEach((n, i) => { if (lines.children[i] !== n) lines.insertBefore(n, lines.children[i] ?? null); });
    while (lines.children.length > nodes.length) lines.lastElementChild.remove();
    const t = totals(cart);
    const status = document.getElementById("status"); // n'annoncer que ce qui change
    const said = `${t.units} créations : ${cents(t.subtotal)} € HT, TVA ${cents(t.vat)} €, total ${cents(t.total)} € TTC.`;
    if (status.textContent !== said) status.textContent = said;
    document.getElementById("delivery").textContent = `Livraison prévue le ${longDate(addDays(new Date(), 3))}.`;
    try { localStorage.setItem("cart", JSON.stringify(cart)); } catch {}
  }
  render();
  // Comme la page HoloCode qui lit l'heure : se tenir à jour à chaque minute (minuit change la date).
  setInterval(render, 60_000);
}
```

Vérifié : la logique de la version web, jouée dans Node avec des dates fixées (`node essais/p10/essai-panier.mjs`), donne neuf « OK » : 129,99 € HT, TVA 26,00 €, 155,99 € TTC ; 19,99 € → TVA 4,00 € ; trois barques → « 3 / 59,97 » ; 7 octobre 2026 + 3 → « samedi 10 octobre 2026 » ; 30 octobre + 3 → « lundi 2 novembre 2026 » ; 28 février 2028 + 1 → « mardi 29 février 2028 » ; un total énorme s'arrête à 1 000 000 000. **Non vérifié** : la page web n'a pas été ouverte dans un navigateur. Remarque honnête : un programmeur web prudent compte aussi en centimes entiers ; l'avantage de HoloCode n'est pas l'exactitude du calcul, mais qu'on ne puisse pas se tromper (pas de virgule possible, un arrondi écrit une fois dans le moteur, le même sur le serveur).

## Par couche

- **Langage** : `computed: Computed(…)` ; `Sum(…)` (de valeurs, ou d'une liste avec `of:` et `times:`) ; `Percent(taux, of:)` ; `today`, `AddDays`, `DaysUntil` ; le format `{x:date}` ; `Status` (partagé avec la piste 1) ; plus tard `Length` ; pour les modules, `input: [a, b]` et `output: [c, d]`.
- **Moteur (Rust)** : calculer les valeurs dans l'ordre de leurs dépendances, et refuser une boucle à la lecture, avec les noms ; ne jamais les ranger dans l'état (comme `count` et `total`, `state.rs:916-937`) : rien de gardé ni de falsifiable par le navigateur ; des entiers bornés de 0 à 1 000 000 000 ; l'arrondi « 0,5 monte » écrit une fois ; une date = un nombre de jours depuis le 1er janvier 1970 (le calcul inverse existe déjà, `state.rs:981-996`) ; lire un texte « AAAA-MM-JJ » ; écrire une date en mots dans la langue de la page (`format.rs`) ; laisser `count`, `total` et les valeurs calculées servir de quantité, comme l'heure aujourd'hui (`state.rs:1110-1119`, `:949-955`) ; les montrer dans `?values`.
- **Enveloppe navigateur** (`page-engine.js`) : réparer la boîte (D0, fait par la PR 141) ; un essai qui la joue comme la page (pas encore écrit) ; peut-être garder un fil ouvert par module, si la mesure montre que le démarrage coûte cher (aujourd'hui un fil neuf à chaque `run`, `page-engine.js:1197`) ; annoncer un `Status`. La page se tient déjà à jour chaque minute quand elle lit l'heure (`page-engine.js:343-354`).
- **Services serveur** : le même moteur en Rust recalcule le total d'une commande à partir des gestes, sans croire la page (c'est le principe de `proposals/Claude/serveur-et-comptes-2026-10-07.md`) ; la page fabriquée d'avance prend l'heure du serveur (`HOLO_NOW`, sinon le temps universel, `src/bin/holo.rs:120-129` ; le serveur de démonstration donne toujours `HOLO_NOW`, d'après son horloge : `moteur/outils/server.mjs:62-66`) : autour de minuit, la date fabriquée peut différer d'un jour de celle du visiteur jusqu'au démarrage du moteur.

## Dépendances

- **Piste 1** : la liste calculée et `Sum` d'un panier reçu du serveur. Une seule notion pour les deux pistes, à décider ensemble.
- **Piste 2** (formulaires) : une date choisie qui sert dans un calcul ; une commande envoyée, que le serveur recalcule (recette X01).
- **Piste 6** (interactions) : les annonces (`Status`).
- **ADR touchées** : ADR-015 (les appels de calcul sont déjà permis), ADR-023 (`sum` écarté), ADR-043 (les expressions écartées, entiers et centimes), ADR-045 (« Restent à faire » des modules), ADR-060 (le défaut de la boîte).
- **Le serveur en Rust** : pour qu'un total serve à une vraie commande.

## Coût

- **Moteur** (estimation) : réparer la boîte : une dizaine de lignes (fait par la PR 141 : six lignes changées dans le texte de la boîte, plus deux de commentaire), plus un essai d'une quarantaine (pas encore écrit) ; `count` et `total` comme quantité, et le message : 20 à 40 lignes ; le cœur des valeurs calculées (`Computed`, l'ordre, les boucles refusées, `Sum`, `Percent`, la lecture dans les textes, `If`, `When` et les demandes) : 350 à 500 lignes ; les dates (`today`, `AddDays`, `DaysUntil`, lire « AAAA-MM-JJ », `{x:date}` en français et en anglais) : 150 à 250 lignes ; les modules à plusieurs nombres (page, moteur, un module d'exemple, une leçon) : 150 à 250 lignes. En tout, environ 700 à 1 050 lignes.
- **Poids transféré** : **mesure** : moteur léger de 156 511 octets en Brotli pour 9 790 lignes de Rust hors essais, soit environ 16 octets par ligne (mêmes commandes que pour la piste 1). **Estimation** : + 11 à 17 Ko compressés. Partagée avec la piste 1 si la notion est commune (le cœur des valeurs calculées ne se paie qu'une fois).
- **Temps de calcul** : **mesure** (Node, PC, moteur léger) : l'arbitre seul (`arbitrate`) coûte aujourd'hui 0,3 ms pour 12 fiches et 1 ms pour 100 (mesure N2 de la piste 1, refaite à la relecture : 0,298 et 0,976 ms). Un geste qui change une ligne coûte en plus le redessin des lignes (0,53 et 2,67 ms) et les conditions (0,23 et 0,65 ms), mesurés à part. **Estimation** : recalculer 32 valeurs au plus sur des listes de 100 éléments ajoute moins de 0,2 ms. Le démarrage d'un module (un fil neuf, plus la préparation du WebAssembly) : **non mesuré** ; estimation de 5 à 30 ms sur un PC, davantage sur un téléphone. Téléphone : **aucun appareil ici, non mesuré**.
- **Travail** (estimation) : réparer la boîte : un quart de séance (fait par la PR 141, reste l'essai automatique) ; valeurs calculées et pourcentage : 2 séances ; dates : 1 séance ; modules à plusieurs nombres : 1 à 1,5 séance. Environ 4 à 5 séances, dont 1,5 commune avec la piste 1.

## Accessibilité, déterminisme, budgets

- **Annonces** : un total qui change est lu une fois, poliment (`Status`), jamais à chaque minute. La version web ne réécrit la zone que si le texte change ; le moteur devra faire de même.
- **Des dates en mots** : « samedi 10 octobre 2026 » se lit sans ambiguïté ; « 10/10/2026 » change de sens entre la France et les États-Unis. Le format `{x:date}` écrit des mots, dans la langue de la page.
- **Les nombres** : « 1 234,50 » s'écrit avec une espace fine insécable (`format.rs:38`). Sa lecture au TalkBack reste à vérifier.
- **Clavier et focus** : « Un de plus » change la ligne, donc la refait aujourd'hui : le focus est perdu (piste 1, point 5). La réparation de la piste 1 vaut ici aussi.
- **Déterminisme** : des entiers seulement ; un arrondi écrit une fois ; un ordre de calcul fixé par les dépendances ; l'heure est une entrée (donnée par la page, ou par `HOLO_NOW` pour `holo html`) ; un essai dans Node peut la fixer (`set_now`, que la page emploie déjà), mais pas encore `holo test` : il ne lit pas `HOLO_NOW` (`src/bin/holo.rs:48-77` répond avant `:120-129`) et n'a aucune ligne pour régler l'heure (`tools.rs:184-204`) ; il joue toujours le jeudi 1er janvier 2026 à 0 h 00 (`state.rs:967` ; `relecture-1-10/year-qty.test` : « ok » avec `best = 2026`, `w = 4`, `h = 0`, même avec `HOLO_NOW` donné). Il faudra une ligne de plus dans `holo test` pour essayer les dates ; un module ne voit ni l'heure ni le réseau (ADR-045). Le même total doit sortir du moteur natif, du moteur WebAssembly et du serveur.
- **Budgets** (propositions, à mesurer) : 32 valeurs calculées au plus, comme les valeurs déclarées ; une valeur calculée bornée à 1 000 000 000, avec un avertissement dans `?values` quand elle touche la borne ; les bornes des modules inchangées (10 ms à 5 s, 64 KB à 16 MB).

## Recette qui peut échouer

| N° | Essai | Attendu | Échec si |
|---|---|---|---|
| C1 | Panier de départ (19,99 € et 110,00 € HT) | TVA 26,00 €, total 155,99 € ; pour 19,99 € seul, TVA 4,00 € | 25,99 € ou 3,99 € (**échoue aujourd'hui** : mesuré, 2599 et 399) |
| C2 | « Un de plus » sur La barque, deux fois | « 4 créations : 169,97 € HT », sans autre geste | il faut un bouton « Calculer » (**échoue aujourd'hui** pour un panier en liste, `Repeat(over: cart)` ; un panier écrit à la main le réussit en recopiant le calcul dans chaque règle, `relecture-1-10/sans-bouton.holo`) |
| C3 | Le même panier montré dans le monde d'un point (comme F05) | les mêmes nombres, par le même calcul | deux calculs différents, ou un écart d'un centime |
| C4 | `HOLO_NOW=2026,10,7,3,10,0`, puis 30 octobre 2026, puis 28 février 2028 (plus un jour), puis `lang: "en"` (avec `holo html`, ou un `holo test` qui saura régler l'heure : aujourd'hui il ne lit pas `HOLO_NOW`) | « samedi 10 octobre 2026 », « lundi 2 novembre 2026 », « mardi 29 février 2028 », « Saturday 10 October 2026 » | une autre date, ou la date du serveur affichée au visiteur après le démarrage du moteur |
| C5 | `Computed(a: Sum(b, 1), b: Sum(a, 1))` | refusé à la lecture, avec les deux noms | accepté, ou une page qui se fige |
| C6 | 100 lignes à 999 999 999 centimes | 1 000 000 000, et un avertissement dans `?values` | un nombre plus grand (aujourd'hui, `total` peut dépasser : `state.rs:927-929`) |
| C7 | Écrire un faux `subtotal` dans le stockage du navigateur, recharger | ignoré : le moteur recalcule | le faux total s'affiche |
| C8 | Leçon 69 dans Chrome : « Calculer la somme », puis la boucle | « Le module a rendu : 5 050 » ; la boucle arrêtée après deux secondes | rien, ou « arrêtée » aussitôt (**échouait** à `1119361`, reproduit dans Node ; sur `9df3b2b`, 5 050 d'après le journal ; la boucle : non vérifié à la relecture) |
| C9 | L'essai automatique de la boîte : le code de `page-engine.js` joué avec le message de la page | `{ start }` puis `{ ok: true, output: 5050 }` | `ok: false` (**échouait** à `1119361` ; sur `9df3b2b`, la sonde rend `{ start }` puis `output: 5050`, mais l'essai automatique n'existe pas encore) |
| C10 | 1 000 paniers tirés d'une graine fixe, calculés dans `holo test`, dans Node (WebAssembly) et par la version web | les mêmes totaux, au centime | un seul écart |
| C11 | Au TalkBack, « Un de plus » | le nouveau total lu une fois | rien n'est lu, ou la phrase est lue à chaque minute |
| C12 | Un module à deux nombres (`weight`, `zone`) qui boucle, puis qui demande trop de mémoire | arrêté par son temps, puis par son plafond ; la page répond pendant ce temps | la page se fige, ou un faux succès |

## Objection

**La meilleure raison de ne pas le faire : chaque fonction est un mot, et les formules sont la pente vers le code.** ADR-023 et ADR-043 ont refusé `sum(price)` et les expressions pour cette raison. Les règles d'argent (TVA par ligne ou sur le total, arrondis selon le pays, plusieurs monnaies) appartiennent au serveur, pas à la page. Les dates ouvrent un puits sans fond (fuseaux horaires, changements d'heure, calendriers). Et élargir les modules agrandit la surface d'attaque. Le faire autrement : une règle qui suit chaque changement (option A3), un réglage d'arrondi sur `div` (option B3), et les totaux calculés par le serveur.

Ma réponse : une valeur calculée sans signes garde l'esprit de HoloCode (rien de caché, tout se lit au même endroit, le moteur vérifie, et le serveur refait le même calcul). Mais elle ajoute une notion au langage : c'est à Yocthan de peser. La réparation de la boîte, elle, n'a pas d'objection.

## Expérience requise

1. **Réparer la boîte et rejouer la leçon 69 dans Chrome** (somme, boucle arrêtée après deux secondes, mémoire refusée), avec une capture pour le journal. Fait en partie par la PR 141 : le journal dit la somme (5 050) vérifiée dans Chrome sans fenêtre ; il ne dit rien de la boucle ni de la mémoire.
2. **Montrer à Yocthan le même panier écrit trois fois** : avec les demandes d'aujourd'hui (`essais/p10/aujourdhui.holo`, déjà fait), avec les valeurs calculées (option A1), avec une règle qui suit chaque changement (option A3). Lui demander laquelle il lit et corrige sans aide.
3. **Un prototype isolé, sur une branche, non fusionné**, des valeurs calculées avec `Sum` et `Percent` ; mesurer le coût d'un geste dans Node, puis dans Chrome, puis sur le téléphone de Yocthan. Sans téléphone : « non mesuré ».
4. **Un banc de déterminisme** : 1 000 paniers tirés d'une graine, comparés entre le moteur natif, le moteur WebAssembly et la version web (essai C10).
5. **Mesurer le démarrage d'un module** (fil neuf et préparation) sur le PC et sur le téléphone, avant de décider s'il faut garder un fil ouvert.
6. **Les dates autour de minuit** : changer la date de l'appareil et `HOLO_NOW`, et regarder ce que montrent la page fabriquée et la page vivante.

## Mises à jour de documents à prévoir

À lister, pas à faire :

- `docs/01-holocode/GUIDE.md` : § 6 bis (`:545`, « pas de centimes » ; `Prices` face à `Sum`) ; § 6 sedecies (valeurs calculées, `Percent`, les dates) ; § 6 duodevicies (modules : la réparation ; plusieurs nombres) ; § 10 aide-mémoire (`computed`, `Computed`, `Module(input: […])`) ; § 11.
- **Leçons** : la 69 à revérifier après la réparation ; nouvelles leçons, une notion chacune : « une valeur calculée », « un pourcentage exact », « une date dans trois jours », « un module à plusieurs nombres ».
- `docs/01-holocode/NOMS.md` : les nouveaux mots ; la ligne `{count}`, `{total}` (`:180`).
- `docs/01-holocode/COMPARAISON-WEB.md:155-156`, `:163-167`.
- `docs/01-holocode/TABLEAU-WEB.md:618-627` : ajouter une ligne « valeurs dérivées » (`computed`, un getter) ; revoir les 40 % des modules après la réparation ; republier la page en ligne tenue pour Yocthan.
- `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md:373`.
- `docs/06-journal/JOURNAL.md` : le défaut de la boîte, sa cause (PR 136), la commande qui le reproduit, la réparation (fait par la PR 141, entrée du 2026-10-07 « Deux pannes de la traduction en anglais, réparées », sans la commande de la sonde).
- `ADR-045` et `ADR-060` : une note datée sur l'épreuve de la leçon 69 (par Claude, sans changer leur statut).
- Une fiche de décision proposée, commune avec la piste 1 (« valeurs et listes calculées »), après l'accord de Yocthan.

Relu le 2026-10-07 : 33 corrections.
