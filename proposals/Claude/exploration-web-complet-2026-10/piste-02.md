# Piste 2 — Formulaires complets

> Statut : EXPLORATION. Avis de Claude, pas une décision.
>
> **Réparé le 2026-10-07, après cette exploration** : Échap ferme de nouveau une fenêtre quand la page écoute `Key.escape` (PR 145).

## Ce que Codex demandait

Sa ligne, dans l'issue #82 (texte transmis par Yocthan le 2026-10-06) :

> | **2** | **Formulaires complets** : choix, texte long, dates, fichiers, validation, envoi, attente, erreur et confirmation. | Envoyer un message, une inscription ou une commande. Aujourd'hui, saisir une valeur locale ne l'envoie pas. |

Et, plus bas dans le même texte : « Les comptes, paiements et commandes demandent aussi un serveur. Le langage peut décrire la demande et afficher sa réponse ; le serveur doit vérifier les droits, valider les données et enregistrer le résultat. »

## État vérifié (main, 7a48def, 2026-10-07)

**Franchement : la moitié de la piste est faite.** Envoyer un message marche déjà, de bout en bout, sur le serveur local. Ce qui manque, c'est tout ce qui rend un formulaire *sûr* : la vérification des champs, l'annonce des erreurs, l'envoi sans doublon, et l'envoi d'une commande.

Remarque : `main` est passé à `1119361` pendant cette exploration (PR 139 : un fichier de thème se vérifie seul). Ce changement ne touche aucune ligne citée ici (seulement `lib.rs`, `bin/holo.rs`, des tests à la fin de `flat.rs` et le journal) ; les numéros de ligne ci-dessous sont ceux de `7a48def`.

### Ce qui existe

| Demande de Codex | Où c'est | Décision, leçon |
|---|---|---|
| Choix | `Choice(value:, label:, options:, menu:)` : boutons ronds dans un `fieldset`, ou liste déroulante (`moteur/src/flat.rs:792-813`) ; la valeur n'accepte que ses options (`moteur/src/state.rs:780-787`) | `ADR-038`, leçon 44 |
| Texte long | `Input(lines:)` → `textarea` (`flat.rs:745-758`) | `ADR-038`, leçon 43 |
| Dates | `Input(type: date \| time \| color)` (`flat.rs:768-774`) ; la page n'accepte que le bon format (`state.rs:767-778`) | `ADR-042`, leçon 60 |
| Fichiers | `Input(type: file, accept:, max:)` dans un `Form` (`flat.rs:759-767`) ; vérifié par la page (`moteur/web/page-engine.js:1150-1167`), puis par le serveur, qui lit la sorte dans les octets (`moteur/outils/server.mjs:221-250`). Défaut vu à la relecture (lu dans le code, pas rejoué) : le fichier est rangé dans `messages/files/<page>/` (`server.mjs:228`), mais le message note le chemin `fichiers/<page>/…` (`server.mjs:249`), un reste de la traduction d'`ADR-060` : ce chemin ne mène à rien | `ADR-059`, leçon 76 |
| Envoi | `Form(name:)` et `Contact.send` (`flat.rs:1088-1109`, `moteur/src/rules.rs:39`) ; la page envoie les valeurs de ses champs en JSON (`state.rs:794-833`, `page-engine.js:1124-1146`) ; le serveur range le message dans `messages/` (`server.mjs:252-258`) | `ADR-042`, leçon 64 |
| Erreur et confirmation | les signaux `sent` et `failed` (`rules.rs:14`, `page-engine.js:1145`) ; l'auteur montre un message par `If`, ou ouvre une `Dialog` | `ADR-042`, leçons 63, 64 |
| Attente | rien d'automatique, mais l'écriture existante suffit (essai ci-dessous) ; un seul envoi à la fois par formulaire (`page-engine.js:1123-1128`) | — |
| Une page qui ne recharge jamais | l'envoi natif est bloqué (`moteur/web/page.html:238`, `page-engine.js:1424-1426`) | `ADR-042` |

**Mesure : l'attente, l'erreur et la confirmation s'écrivent aujourd'hui.** Fichier d'essai `essais-2-6-7/contact-existant.holo` (trois valeurs, trois règles) et son essai écrit :

```text
$ holo.exe check contact-existant.holo
ok
$ holo.exe test contact-existant.holo contact-existant.test
contact-existant.test : ok, 14 ligne(s) jouée(s)
```

L'essai tape un nom, touche « Send », simule `Contact.failed`, puis `Contact.sent`, et vérifie chaque valeur.

### Ce qui manque encore (vérifié par `holo check`, sondes `essais-2-6-7/sondes.sh`)

1. **La vérification des champs.** `Input` n'accepte que `name, value, label, max, lines, type, accept` (`moteur/src/blocks.rs:53`). Refusés : `required` (sonde P2-02), `type: email` (P2-03 : « attend date, time, color ou file »), `min:` sur un texte ou une date (P2-04, P2-12), une aide sous le champ (P2-05). Le formulaire est fabriqué avec `novalidate` (`flat.rs:1106`) : le navigateur ne vérifie rien non plus. Ce qui est vérifié aujourd'hui l'est en silence, par l'arbitre : la longueur d'un texte (coupée à `max`, sinon à 80 ou 1 000 caractères, `state.rs:788-790`), un nombre borné, une option de `Choice` (`state.rs:780-787`), le format d'une date, d'une heure ou d'une couleur (`state.rs:767-778`). Seuls la sorte et la taille d'un fichier sont dites au visiteur (`page-engine.js:1162-1165`). Rien n'est obligatoire.
2. **Des erreurs reliées au champ et annoncées.** Aucun `aria-invalid`, aucun `aria-describedby` (HTML mesuré, sonde P2-17 : `<input type="text" maxlength="80" value="" data-bind="name">`). Les seules zones annoncées au lecteur d'écran servent aux changements de vue (`page.html:119`, `page-engine.js:789`, `812`, `816-820`) et à la panne du moteur (`page.html:121`, `role="alert"`) : un « Merci » ou un « Échec » montré par un `If` n'est jamais annoncé (WCAG 4.1.3).
3. **Envoyer avec Entrée.** L'envoi natif est bloqué, aucun signal ne le remplace (P2-06 : « signal inconnu « submit » ») ; `On(Key.enter, …)` est accepté (P2-07) mais ignoré quand on écrit dans un champ (`page-engine.js:1418-1419`).
4. **Une commande.** Seules les valeurs que présentent les champs du formulaire partent (`state.rs:800-808`). Les quantités du panier, ou une liste (`Repeat(over:)`), ne partent pas (P2-08 refusé ; P2-09 accepté mais rien n'est envoyé). Pour joindre une quantité, il faut en faire un champ modifiable (P2-16).
5. **Montrer un champ selon un choix.** Une valeur texte de la page ne se compare qu'au vide (`state.rs:78-79`) : `If(delivery, is: "À domicile")` est refusé (P2-15). Un `If` montre un champ selon un nombre (une case à cocher, un champ de nombre, une glissière), ou selon qu'un texte est vide ou rempli (`If(delivery, not: "")`, accepté) ; jamais selon *l'option* choisie dans un `Choice`.
6. **Doublons et silence.** Aucune clé d'envoi : si la réponse se perd après l'écriture, un nouvel essai écrit une seconde ligne (`server.mjs:252-256` ajoute toujours). Aucune durée maximale : `fetch` n'a pas de délai (`page-engine.js:1141`) ; un serveur muet laisse l'attente sans fin. `failed` ne dit jamais pourquoi (`page-engine.js:1142-1145`).
7. **Le serveur croit la page.** Il vérifie la taille (`server.mjs:203-207`) et la forme du JSON (`server.mjs:217-219`), pas les champs : un envoi forgé, avec n'importe quel nom de formulaire et n'importe quelle clé, est écrit (jusqu'à 16 Ko), pour toute page qui existe sous `exemples/` (`server.mjs:363-367`). Seuls les fichiers sont vérifiés par le moteur (`holo files`, `server.mjs:221-243`).
8. **Sans JavaScript, rien.** Les champs n'ont pas d'attribut `name`, le bouton est `type="button"` (P2-17) : le formulaire est inerte. Il ne prétend rien avoir envoyé, mais il ne dit rien non plus.
9. **Le but d'un champ** (`autocomplete="name"`, `"email"`) : absent (P2-17). C'est une exigence WCAG 1.3.5 (niveau AA) pour les champs qui demandent des informations sur le visiteur.
10. **Regrouper des champs** (`fieldset`, `legend`) : absent, sauf dans `Choice` (`docs/01-holocode/TABLEAU-WEB.md:528`).

### Documents en retard

- `docs/01-holocode/GUIDE.md:816` : « Limites : pas de liste de choix, pas d'envoi à un serveur » (les deux existent : `ADR-038`, `ADR-042`).
- `GUIDE.md:1692` : l'aide-mémoire de `Input` oublie `type: file` et `accept`.
- `GUIDE.md:1801` : « l'envoi d'un fichier » est rangé dans ce qui n'existe pas (fait : `ADR-059`).
- `docs/01-holocode/COMPARAISON-WEB.md:81` (`form` : « manque », alors que `:88` dit l'envoi fait), `:84` (« date, bouton radio, fichier… : manque », alors que `:87` et `:90` les disent faits), `:99` (`output`, `progress` : « manque », alors que `:91` dit `Progress` fait), `:160` (« pas de liste, pas d'envoi »), `:187` (verdict : « Pas de formulaire, pas de panier »). L'en-tête dit « État au 2026-10-03 » (`:3`).
- `docs/01-holocode/NOMS.md:228` : `form, input, textarea, select, label` dans « Pas encore là ».
- `docs/02-gouvernance/adr/ADR-042-lot-5-html-utile-et-formulaire.md:44` : le champ « fichier » est encore « pour plus tard » (fait depuis `ADR-059` ; une ADR acceptée ne se réécrit pas, mais `ADR-059` pourrait le rappeler).
- `TABLEAU-WEB.md:518` donne `Form` à 90 % : c'est généreux sans vérification ni erreurs annoncées. Avis : environ 60 % (estimation).
- `TABLEAU-WEB.md:524` dit : « Le courriel et le mot de passe viendront avec des comptes » (et `ADR-042:44` range les champs e-mail « pour plus tard ») : l'option A1 (`type: email` dès maintenant) change ce plan ; à écrire dans le tableau si elle est retenue.

## Le scénario du site de référence

**Le cas de Codex :** l'extension X01 du cahier, « Envoi réel d'un formulaire ; confirmation serveur, validation, coupure et reprise sans doublon », état `BLOQUÉ` (`proposals/GPT5.6/site-reference-2026-10-06/RECETTE.md:90`) ; et dans le cahier : « Un formulaire envoie une demande, le serveur confirme une seule réception ; coupure et nouvelle tentative ne créent pas un doublon » (`README.md:132`). Le premier passage de recette le laisse bloqué (`exemples/site-reference/RECETTE-2026-10-06.md:24`).

**La tâche concrète sur le site construit :** le panier (`exemples/site-reference/panier.holo:59-63`) a déjà un prénom, l'emballage cadeau et les cartes de vœux, mais « Cette commande fictive est au nom de {nom} » : rien ne part. Il s'agit d'en faire une **demande de réservation** :

1. le visiteur écrit son nom et son courriel (obligatoires, courriel bien formé), choisit « À domicile » ou « À l'atelier » ; l'adresse n'est demandée que pour « À domicile » ;
2. la demande emporte les trois quantités du panier ; le serveur recalcule le total lui-même ;
3. pendant l'envoi, la page le montre et le dit ; à l'arrivée, « Merci » est montré et annoncé ; en cas d'échec, la saisie reste, et un nouvel essai ne crée pas de doublon.

**Ce qu'il faut pour la faire :** la vérification des champs (manque 1 et 2), la comparaison à un texte (manque 5), les valeurs jointes (manque 4), la clé d'envoi et le délai (manque 6), la vérification par le serveur (manque 7). L'annonce (manque 2) sert aussi à la piste 6.

## Options comparées

### A. Vérifier les champs

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **A1. Des réglages sur le champ** | `Input(value: mail, label: "E-mail", type: email, required: true)`, `Input(…, min: 2)` | `required`, `type="email"`, `minlength`, `min` ; messages et `aria-*` écrits à la main en JS | `TextFormField(validator: …)` dans un `Form`, `formKey.currentState!.validate()` | les mots du web ; une seule déclaration, que le serveur peut relire avec le même moteur | les messages doivent venir du moteur (ceux du navigateur disparaissent vite et se lisent mal au lecteur d'écran) |
| A2. Des règles à part | `Check(mail, is: email, message: "…")` dans `rules` | du JS | les fonctions `validator` | messages libres | un bloc de plus ; la règle s'éloigne de son champ ; plus long |
| A3. Rien de nouveau | `If(name, not: "", children: [ Button(…) ], else: [ Text("Écris ton nom.") ])` (accepté, sonde P2-13) | — | — | aucun mot | pas de format de courriel ; le bouton disparaît (le focus se perd) ; rien n'est annoncé ni relié au champ |

### B. Annoncer ce qui arrive (confirmation, erreur, attente)

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **B1. Un réglage sur `If`** | `If(sent, is: 1, announce: true, children: [ P("Merci") ])` | une zone `role="status"` / `aria-live`, remplie en JS | `SemanticsService.announce(…)` | un mot ; réutilise la zone qui existe (`page.html:119`) ; sert aussi à la piste 6 | l'auteur doit y penser |
| B2. Tout annoncer seul | le moteur lit tout `If` qui apparaît | — | — | rien à écrire | bavard : un lecteur d'écran lirait chaque changement de panier |
| B3. Une fenêtre | `Dialog` + `Thanks.open` (existe) | `dialog.showModal()` | `showDialog` | existe ; le focus entre dans la fenêtre | lourd pour un simple « Merci » ; la fenêtre n'a pas de nom (piste 6, mesuré) |

Pour l'**attente**, deux options : **B4. le moteur la montre seul** (le bouton qui envoie prend le signe d'attente qui existe déjà avant l'arrivée du moteur, `page.html:109-111`, plus `aria-disabled` et une annonce « Envoi en cours ») ; ou B5. l'auteur l'écrit, comme aujourd'hui (`sending.set(1)`, mesuré plus haut). B4 évite le piège de B5 : cacher le bouton qui a le focus.

### C. Envoyer avec Entrée

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **C1. Automatique** | rien : Entrée dans un champ d'une ligne touche le bouton qui envoie ce formulaire (le moteur le connaît par les règles) | l'« envoi implicite » d'un `form` avec un bouton `type="submit"` | `onFieldSubmitted` | aucun mot ; le geste que tout le monde connaît | un formulaire à deux boutons d'envoi doit en désigner un (rare) |
| C2. Un signal | `On(Contact.submit, effect: Contact.send)` | l'événement `submit` | `onSubmitted` | explicite | un mot de plus pour un geste attendu partout |

### D. Envoyer une commande (des valeurs qui ne sont pas des champs)

| Option | Écriture HoloCode | HTML/CSS/JS | Flutter | Avantages | Défauts |
|---|---|---|---|---|---|
| **D1. Joindre des valeurs** | `Form(name: Booking, include: [lever, porte], …)` ; une liste aussi : `include: [articles]` | des `<input type="hidden">`, ou un objet ajouté en JS | aucun | court ; le serveur, qui a le même moteur, recalcule `{total}` avec `Prices` : il ne croit jamais un total envoyé | une liste à champs peut peser (100 éléments × 16 champs au plus, `ADR-051`) |
| D2. Des champs cachés | `Input(type: hidden, …)` | `type="hidden"` | — | familier | le défaut du web : invisibles, falsifiables, et l'on y met des prix |
| D3. Des champs modifiables | `Input(value: lever, label: "…", max: 10)` dans le `Form` (accepté, P2-16) | `input type="number"` | — | existe | change l'interface ; pas de liste |

### E. Un champ qui dépend d'un choix

| Option | Écriture HoloCode | HTML/CSS/JS | Avantages | Défauts |
|---|---|---|---|---|
| **E1. Comparer un texte à un texte** | `If(delivery, is: "À domicile", children: [ … ])`, et de même pour `When` | `if (form.delivery.value === "À domicile")` | aucun mot nouveau ; déjà permis sur un champ de ligne (`ADR-057:12`, `If(item.status, is: "late")` dans un `Repeat(over:)`, accepté ; un champ ne peut pas s'appeler `state` : refusé, avec un message qui égare) | une faute de frappe dans le texte : le moteur doit refuser un texte qui n'est pas une option du `Choice` |
| E2. Une case à cocher | `Checkbox(value: home, …)` puis `If(home, is: 1)` (existe) | `checkbox` | existe | seulement deux cas |

### F. Ni doublon ni attente sans fin

| Option | Où | HTML/CSS/JS | Avantages | Défauts |
|---|---|---|---|---|
| **F1. Une clé par envoi, et un délai** | moteur et serveur, rien à écrire | en-tête `Idempotency-Key` (brouillon de l'IETF), `AbortSignal.timeout(15000)` | « coupure et reprise sans doublon » (X01) ; `failed` arrive au bout de 15 s (estimation du bon délai) | le serveur garde les clés récentes (quelques Ko) |
| F2. Le serveur compare le contenu | serveur | — | rien côté page | deux messages identiques voulus seraient fondus |

### G. Le serveur vérifie avec le moteur

| Option | Où | Avantages | Défauts |
|---|---|---|---|
| **G1. `holo files` étendu aux champs** | le serveur demande au moteur les champs de chaque formulaire, leurs bornes et leur sorte ; il refuse un champ inconnu, trop long, mal formé, ou obligatoire et vide | la même règle des deux côtés, sans rien réécrire ; c'est déjà le chemin de `ADR-059` | un appel au moteur par envoi (aujourd'hui un processus lancé : quelques millisecondes, estimation) |
| G2. Le futur `holo serve` | rejoue le geste avec le même arbitre (`proposals/Claude/serveur-et-comptes-2026-10-07.md`) | la cible propre | attend les réponses de Yocthan à cette proposition |

### H. Sans JavaScript

| Option | Avantages | Défauts |
|---|---|---|
| H1. Un vrai `<form method="post">`, des attributs `name`, un bouton `type="submit"` ; le serveur accepte aussi l'envoi classique et répond par une page « Merci » | marche moteur absent, ou JavaScript coupé (cas F01 du cahier) | le serveur fabrique une page de réponse ; les règles `sent`/`failed` ne jouent pas sans moteur |
| **H2. Rien (aujourd'hui)** | aucun travail | un bouton inerte, sans un mot |

### Noms (ADR-016)

| Nom proposé | Sens sur le web | Sens dans Flutter | Risque de confusion |
|---|---|---|---|
| `required` | attribut `required` : le même sens | pas de `required` sur un champ ; en Dart, `required` marque un paramètre nommé obligatoire | faible |
| `type: email` | `input type="email"` : le même sens | `keyboardType: TextInputType.emailAddress` (le clavier seulement) | faible ; suit `type: date`, `file` |
| `min:` (sur un texte : la longueur) | `min` borne un nombre ou une date ; la longueur minimale s'écrit `minlength` | `TextField` a `maxLength`, pas de longueur minimale | **moyen** : un développeur web lira un nombre. Mais HoloCode a déjà choisi `max:` pour la longueur d'un texte (`GUIDE.md:811`) ; `min:` en est le pendant |
| `include:` (sur `Form`) | aucun attribut ; `@include` en SCSS ajoute des styles | aucun | faible ; alternative `values:` (comme `FormData.values()`), mais on lirait « seulement ces valeurs » |
| `announce:` | aucun attribut ; c'est `aria-live` | `SemanticsService.announce` : le même sens | faible |
| `autofill:` (but du champ : `name`, `email`, `tel`, `address`) | attribut `autocomplete` | `autofillHints` : le même sens ; mais `Autocomplete` est un widget de suggestions | faible ; `autocomplete` serait mal lu par un développeur Flutter |
| `help:` (texte d'aide sous le champ) | aucun ; relié par `aria-describedby` | `InputDecoration(helperText:)` ; `hintText` est le texte *dans* le champ | faible ; éviter `hint` |

## Recommandation

Dans cet ordre, de ce qui ne demande aucune décision de langage à ce qui en demande une :

1. **Mettre les documents à jour** (liste plus bas).
2. **F1 et G1 : sans doublon, avec un délai, vérifié par le serveur.** Rien ne change pour l'auteur ; c'est la moitié de X01.
3. **A1 + B4 + C1 : la vérification par des réglages du champ** (`required`, `type: email`, `min:`), les messages écrits par le moteur dans la langue de la page, sous le champ, au moment de l'envoi, effacés dès que le champ devient juste ; le focus sur le premier champ faux ; l'attente montrée par le moteur ; Entrée qui envoie.
4. **E1 : comparer un texte à un texte**, dans `If` et `When` (partagé avec la piste 6).
5. **B1 : `announce: true` sur `If`** (partagé avec la piste 6).
6. **D1 : `include:`** pour la commande, avec le total recalculé par le serveur.
7. Plus tard : `autofill:`, `help:`, regrouper des champs, et H1 si la recette F01 le demande.

Une règle à décider avec A1 : **un champ caché par un `If` n'est ni vérifié ni envoyé** (sinon une adresse vide bloquerait « À l'atelier »). Aujourd'hui, il part quand même : `state.rs:800-808` parcourt tout le formulaire, `If` compris.

## Exemple d'auteur

Les lignes marquées `// proposé` sont une **écriture proposée** : elle n'existe pas. Le reste existe. L'attente, l'envoi par Entrée, les messages d'erreur, la clé d'envoi et le délai seraient faits par le moteur, sans rien écrire (proposé).

```holo
Page(
  title: "Réserver — L'atelier des mondes",
  lang: "fr",
  state: State(lever: 2, porte: 1, name: "", mail: "", delivery: "", address: "", gift: 0, sent: 0, failed: 0),
  prices: Prices(lever: 120, porte: 90),
  children: [
    H1("Réserver vos tableaux"),
    P("{count} tableaux, {total} euros."),
    Form(name: Booking, include: [lever, porte], children: [                                       // proposé : include
      Input(value: name, label: "Votre nom", required: true, autofill: name),                      // proposé : required, autofill
      Input(value: mail, label: "Votre courriel", type: email, required: true, autofill: email),   // proposé : type: email, required, autofill
      Choice(value: delivery, label: "Livraison", options: ["À domicile", "À l'atelier"], required: true),     // proposé : required
      If(delivery, is: "À domicile", children: [                                                    // proposé : comparer un texte
        Input(value: address, label: "Adresse de livraison", lines: 3, required: true, autofill: address),   // proposé : required, autofill
      ]),
      Checkbox(value: gift, label: "Emballage cadeau"),
      Button(name: Send, text: "Réserver"),
    ]),
    If(sent, is: 1, announce: true, children: [ P("Merci, votre réservation est arrivée.") ]),        // proposé : announce
    If(failed, is: 1, announce: true, children: [ P("La réservation n'est pas partie : réessayez.") ]),    // proposé : announce
  ],
  rules: [
    On(Send.tap, effect: [failed.set(0), Booking.send]),
    On(Booking.sent, effect: sent.set(1)),
    On(Booking.failed, effect: failed.set(1)),
  ],
)
```

Vérifié : ce fichier est refusé aujourd'hui, à la première écriture proposée, ce qui confirme que ces mots n'existent pas :

```text
$ holo.exe check essais-2-6-7/piste-02-exemple.holo
essais-2-6-7/piste-02-exemple.holo : ligne 9, colonne 25 : « Form » n'a pas de paramètre « include » ; paramètres possibles : name, children
$ holo.exe check essais-2-6-7/piste-02-sans-propose.holo     (le même, mots proposés retirés, « is: "À domicile" » remplacé par « not: "" »)
ok
```

Le reste du fichier est donc de l'écriture d'aujourd'hui, acceptée par le moteur.

**Le même en HTML, CSS et JavaScript**, qui fait exactement la même chose (vérification, messages reliés, focus, Entrée, champ conditionnel, valeurs jointes, attente annoncée, clé d'envoi, délai de 15 s, confirmation et erreur annoncées). Fichier : `essais-2-6-7/piste-02-jumeau.html`.

```html
<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Réserver — L'atelier des mondes</title>
<style>
  .error { color: #b00020; margin: 4px 0 0; }
  .waiting { cursor: progress; animation: pulse .9s ease-in-out infinite alternate; }
  @keyframes pulse { to { opacity: .45; } }
  @media (prefers-reduced-motion: reduce) { .waiting { animation: none; opacity: .6; } }
  .for-reader { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
</style>
</head>
<body>
<main>
  <h1>Réserver vos tableaux</h1>
  <p>3 tableaux, 330 euros.</p>
  <form id="booking" novalidate>
    <p><label for="name">Votre nom</label><br>
      <input id="name" maxlength="80" autocomplete="name" required></p>
    <p><label for="mail">Votre courriel</label><br>
      <input id="mail" type="email" maxlength="80" autocomplete="email" required></p>
    <fieldset id="delivery"><legend>Livraison</legend>
      <label><input type="radio" name="delivery" value="À domicile" required> À domicile</label>
      <label><input type="radio" name="delivery" value="À l'atelier"> À l'atelier</label>
    </fieldset>
    <p id="address-row" hidden><label for="address">Adresse de livraison</label><br>
      <textarea id="address" rows="3" maxlength="1000" autocomplete="street-address" required></textarea></p>
    <p><label><input id="gift" type="checkbox"> Emballage cadeau</label></p>
    <button type="submit">Réserver</button>
  </form>
  <p id="sent" hidden>Merci, votre réservation est arrivée.</p>
  <p id="failed" hidden>La réservation n'est pas partie : réessayez.</p>
  <div id="live" class="for-reader" role="status" aria-live="polite"></div>
</main>
<script>
  const $ = (id) => document.getElementById(id);
  const form = $("booking");
  const say = (text) => { $("live").textContent = ""; setTimeout(() => ($("live").textContent = text), 50); };
  let key = crypto.randomUUID(); // une clé par réservation : un second envoi n'est pas un doublon
  let sending = false;
  form.addEventListener("change", (e) => {
    if (e.target.name === "delivery") $("address-row").hidden = e.target.value !== "À domicile";
  });
  function check(field) {
    const box = field.type === "radio" ? $("delivery") : field;
    const id = `${box.id}-error`;
    let message = "";
    if (!field.closest("[hidden]")) {
      if (field.validity.valueMissing) message = "Ce champ est à remplir.";
      else if (field.validity.typeMismatch) message = "Un courriel s'écrit comme nom@exemple.fr.";
    }
    $(id)?.remove();
    box.removeAttribute("aria-invalid");
    box.removeAttribute("aria-describedby");
    if (message) {
      (field.type === "radio" ? box : field.parentElement).append(Object.assign(document.createElement("p"), { id, className: "error", textContent: message }));
      box.setAttribute("aria-invalid", "true");
      box.setAttribute("aria-describedby", id);
    }
    return !message;
  }
  const fields = () => [$("name"), $("mail"), form.querySelector("input[name=delivery]"), $("address")];
  form.addEventListener("input", (e) => {
    const field = e.target.type === "radio" ? form.querySelector("input[name=delivery]") : e.target;
    if ((field.type === "radio" ? $("delivery") : field).hasAttribute("aria-invalid")) check(field);
  });
  form.addEventListener("submit", async (e) => {
    e.preventDefault(); // la page ne se recharge pas ; Entrée dans un champ arrive ici aussi
    if (sending) return;
    const wrong = fields().filter((f) => !check(f));
    if (wrong.length) return wrong[0].focus();
    sending = true;
    const button = form.querySelector("button");
    button.setAttribute("aria-disabled", "true");
    button.classList.add("waiting");
    $("failed").hidden = true;
    say("Envoi en cours…");
    const values = { name: $("name").value, mail: $("mail").value, delivery: form.delivery.value, gift: $("gift").checked ? 1 : 0, lever: 2, porte: 1 };
    if (!$("address-row").hidden) values.address = $("address").value;
    let arrived = false;
    try {
      const response = await fetch(location.pathname, {
        method: "POST",
        headers: { "content-type": "application/json", "idempotency-key": key },
        body: JSON.stringify({ form: "Booking", values }),
        signal: AbortSignal.timeout(15000),
      });
      arrived = response.ok;
    } catch { /* pas de réseau, serveur muet, ou plus de 15 s */ }
    sending = false;
    button.removeAttribute("aria-disabled");
    button.classList.remove("waiting");
    if (arrived) { key = crypto.randomUUID(); $("sent").hidden = false; say($("sent").textContent); }
    else { $("failed").hidden = false; say($("failed").textContent); }
  });
</script>
</body>
</html>
```

À fonctions égales, il faut **en plus**, côté web, le code du serveur qui revérifie chaque champ et recalcule le total (non écrit ici). Côté HoloCode, le serveur demanderait les mêmes règles au moteur (option G1). Un piège du web que HoloCode n'a pas : un champ nommé « name » cache la propriété `form.name` du formulaire (et un champ « submit » cache `form.submit()`). Mesuré à la relecture, Chrome 154 sans fenêtre, `<form name="Booking">` avec `<input name="name">` : `form.name` rend le champ, pas « Booking ». D'où les `id` partout dans le jumeau.

Mesuré (commande `node mesure-jumeaux.mjs`, compression Brotli qualité 11 comme `server.mjs:90`) : le jumeau pèse **5 052 octets bruts, 1 780 octets compressés, 66 lignes de JavaScript** ; la syntaxe de son script passe `node --check`. Ces chiffres sont ceux du fichier `essais-2-6-7/piste-02-jumeau.html`, qui a des commentaires et des lignes de plus ; le bloc montré ci-dessus (même code) pèse **4 813 octets bruts, 1 670 compressés, 60 lignes de JavaScript** (relecture : `node relecture-2-6-7/mesure.mjs`).

Essayé dans Chrome sans fenêtre, ouvert depuis le disque, sans serveur (`node jumeaux-essai.mjs`) :

```text
2 · Entrée, tout vide : envois → 0 ; focus → name ; erreurs → "name-error : Ce champ est à remplir. | mail-error : Ce champ est à remplir. | delivery-error : Ce champ est à remplir."
2 · courriel « ada@ » : envois → 0 ; focus → mail ; erreurs → "Un courriel s'écrit comme nom@exemple.fr." ; adresse cachée ? → true
2 · tout juste (sans serveur, en file://) : envois → 1 ; message d'échec visible ? → true
```

La clé d'envoi et le délai de 15 s n'ont pas été éprouvés : il faut un serveur qui coupe sa réponse.

## Par couche

- **Langage** : `required`, `type: email`, `min:` (texte et date), `include:`, `announce:` ; la comparaison à un texte dans `If` et `When`. Plus tard : `autofill:`, `help:`.
- **Moteur (Rust)** : `blocks.rs:53` et `state.rs:1204-1256` (la vérification des réglages d'`Input`) pour les nouveaux réglages ; la vérification d'un formulaire (une fonction pure : état + formulaire → liste d'erreurs), utilisée par la page *et* par le serveur ; `state.rs:794-833` pour `include:` et pour écarter les champs cachés ; `state.rs:78-79` pour comparer un texte ; `flat.rs:740-790` pour `required`, `type="email"`, `autocomplete`, la place du message d'erreur.
- **Enveloppe navigateur** (`page-engine.js`) : avant `fetch`, demander les erreurs au moteur, les poser sous les champs avec `aria-invalid` et `aria-describedby`, mettre le focus sur le premier ; le signe d'attente sur le bouton ; la clé d'envoi et `AbortSignal.timeout` ; Entrée dans un champ (au lieu de seulement bloquer l'envoi natif, `page-engine.js:1424-1426`) ; l'annonce, avec `announce()` qui existe (`page-engine.js:816-820`).
- **Services serveur** (`server.mjs`, puis `holo serve`) : `holo files` étendu aux champs ; refuser 400/422 ce qui ne correspond pas ; garder les clés d'envoi récentes et répondre « déjà reçu » sans réécrire ; recalculer le total d'une commande avec le moteur.

## Dépendances

- G1 (le serveur vérifie) dépend de A1 : il relit les mêmes réglages.
- D1 (`include:` d'une liste) dépend des listes à champs (`ADR-051`, fait) et rejoint la piste 1 (données).
- B1 (`announce:`) et E1 (comparer un texte) sont partagés avec la piste 6 : à construire une seule fois.
- La cible G2 dépend de la proposition « serveur et comptes » (quatre questions à Yocthan) ; une inscription avec compte en dépend entièrement.
- Le mot de passe et les comptes restent hors de cette piste (`TABLEAU-WEB.md:524`).

## Coût

| Travail | Moteur | Poids transféré | Travail |
|---|---|---|---|
| F1 clé et délai | ~40 lignes de JS, ~30 lignes du serveur (estimation) | +0,3 Ko compressé (estimation) | 0,5 séance (estimation) |
| G1 vérifié par le serveur | ~80 lignes de Rust, ~40 du serveur (estimation) | 0 | 1 séance (estimation) |
| A1 + B4 + C1 | ~250 lignes de Rust, ~120 de JS (estimation) | +2 à 4 Ko compressés sur le moteur léger (estimation) | 2 séances, leçon et essais compris (estimation) |
| E1 comparer un texte | ~60 lignes de Rust (estimation) | +0,5 Ko (estimation) | 0,5 séance (estimation) |
| B1 `announce:` | ~30 lignes de Rust, ~20 de JS (estimation) | +0,2 Ko (estimation) | 0,5 séance (estimation) |
| D1 `include:` | ~100 lignes de Rust, ~30 du serveur (estimation) | +1 Ko (estimation) | 1 séance (estimation) |

**Le poids d'aujourd'hui, mesuré** (Chrome sans fenêtre sur le PC, serveur local en Brotli, `transferSize` du navigateur ; ce n'est pas un téléphone) :

```text
$ bash poids.sh
64-formulaire.holo, ouvert sans geste : {"total_ko":8.8,"page_ko":8.5,"moteur":false}
27-donnees.holo, ouvert sans geste : {"total_ko":195.2,"moteur":true,"fichiers":"page-engine.js 22.8, holo_engine.js 4.7, holo_engine_bg.wasm 156.8, …"}
```

Un formulaire HoloCode pèse donc **8,8 Ko à l'ouverture**, puis **environ 184 Ko de plus** dès que le visiteur entre dans un champ (le moteur arrive au premier focus, `page.html:239`) ; ces 184 Ko sont mesurés sur la leçon 27, qui charge les mêmes fichiers. Le jumeau web pèse 1,8 Ko compressé (le corps seul, en Brotli qualité 11). Pour comparer à égalité : la page HoloCode est envoyée en Brotli qualité 5 (`server.mjs:392`), soit 8 195 octets pour la leçon 64, plus 250 octets d'en-têtes ; en qualité 11, elle ferait 7 565 octets (relecture : `curl -H 'Accept: text/html'` puis `zlib.brotliCompressSync` de Node, PC). Pour un formulaire de contact seul, le web reste cent fois plus léger ; l'écart se réduit sur un site où le moteur sert déjà ailleurs.

## Accessibilité, déterminisme, budgets

- **Accessibilité** : chaque champ garde son étiquette obligatoire (déjà vrai). À ajouter : l'erreur dite en mots, sous le champ, reliée (`aria-describedby`), avec `aria-invalid` (WCAG 3.3.1, 3.3.3) ; le focus sur le premier champ faux ; les confirmations et les échecs annoncés (WCAG 4.1.3) ; le but des champs (`autocomplete`, WCAG 1.3.5) ; ne jamais cacher le bouton qui a le focus pendant l'attente. Le clavier : Entrée envoie, Tab parcourt, rien ne piège. Moins de mouvement : le signe d'attente devient fixe (`page.html:111` le fait déjà).
- **Déterminisme** : la vérification est une fonction pure de l'état et du fichier : le même fichier et les mêmes saisies donnent les mêmes erreurs, sur la page et sur le serveur. La clé d'envoi est tirée au hasard, mais elle reste hors de l'état de la page : l'arbitre ne la voit jamais. Les messages dépendent seulement de `lang`.
- **Budgets** : rien de plus à l'ouverture (8,8 Ko mesurés) ; le moteur léger grossit de quelques Ko (estimation) ; un envoi reste borné à 16 Ko de valeurs (`server.mjs:156`) ; les clés gardées par le serveur, bornées (par exemple 10 000 clés ou 24 h, estimation à décider).

## Recette qui peut échouer

| Essai | Ce qui doit se passer | Échec si |
|---|---|---|
| R1. Courriel vide, toucher « Réserver » | aucune requête (le journal du serveur et `messages/` ne bougent pas) ; « Ce champ est à remplir. » sous le champ ; `aria-invalid="true"` ; le focus est dans le champ | une requête part, ou le focus reste sur le bouton |
| R2. Écrire « ada@ », envoyer ; puis « ada@exemple.fr » | d'abord le message de format ; puis l'erreur s'efface et l'envoi part | le message reste, ou rien ne part |
| R3. Entrée dans le champ « Votre nom » d'un formulaire juste | un seul envoi | rien ne part, ou deux envois |
| R4. Choisir « À l'atelier », adresse vide, envoyer | l'envoi part, sans adresse dans le message | le formulaire est bloqué par le champ caché, ou l'adresse part |
| R5. Serveur réglé pour écrire puis couper la réponse (comme `HOLO_ENGINE=slow:5000` le fait pour le moteur) ; réessayer | `failed` après 15 s ; au nouvel essai, `messages/…jsonl` a **une** ligne | deux lignes |
| R6. Envoi forgé à la main : `{"form":"Booking","values":{"x":"y"}}`, puis un courriel de 5 000 caractères, puis `required` vide | 400 ou 422, rien d'écrit | une ligne écrite |
| R7. Commande : 2 × 120 + 1 × 90, envoi forgé avec `"total": 1` | le serveur écrit le total qu'il a recalculé : 330 | il écrit 1 |
| R8. TalkBack sur le Galaxy Z Flip (protocole `docs/01-holocode/ESSAI-LECTEUR-D-ECRAN.md`) | l'erreur est lue en arrivant sur le champ ; « Merci » est lu sans déplacer le doigt | silence |
| R9. `holo test` : `type mail "ada@"`, `tap Send`, puis `expect` sur une valeur qui n'a pas bougé | l'essai voit que rien n'est parti (il faudra un mot pour le dire, par exemple `expect Booking = invalid`, à décider) | l'essai ne peut rien vérifier |
| R10. Audit axe-core (`node moteur/outils/accessibility.mjs`) sur la nouvelle leçon, avant et après un envoi raté (l'outil demande d'abord `npm install --no-save playwright axe-core`, `accessibility.mjs:4` : c'est la question en attente) | 0 défaut | un défaut |

R5, R8 et un passage en mode avion au milieu d'un envoi demandent un **vrai téléphone** : pas d'appareil ici.

## Objection

**La meilleure raison de ne pas le faire maintenant :** le serveur n'est pas choisi. Tant que les messages vont dans un dossier du PC de Yocthan, la vérification côté serveur, la clé d'envoi et la commande servent un serveur d'essai ; une partie sera refaite dans `holo serve`. Et chaque mot ajouté alourdit un langage que Yocthan veut court.

**Réponse, à discuter :** la vérification est une fonction pure du moteur ; elle servira telle quelle à `holo serve`. Seule la partie du serveur Node (`server.mjs`) serait jetée, une centaine de lignes. Et les mots proposés sont ceux du web, sauf `include:` et `announce:`.

**Autre façon de faire :** ne rien ajouter au langage, et tout confier au serveur (il refuse, la page affiche « échec »). C'est plus simple, mais le visiteur ne sait pas *quel* champ corriger : c'est le défaut que WCAG 3.3.1 demande d'éviter.

## Expérience requise

1. Écrire la réservation du site de référence avec l'écriture d'aujourd'hui (sans les mots proposés), puis avec : compter les lignes, et surtout noter ce qui reste impossible (le format du courriel, l'annonce, la commande).
2. Faire l'essai R8 au TalkBack sur le téléphone de Yocthan avec un prototype : un message d'erreur relié est-il vraiment lu ?
3. Couper le réseau (mode avion) au milieu d'un envoi, sur le téléphone : vérifier qu'il n'y a ni doublon ni attente sans fin.
4. Montrer à Yocthan les messages d'erreur proposés, en français et en anglais, avant de les figer.
5. Décider avec lui : les noms (`include:` ou `values:`, `min:` ou `minLength:`) ; si un champ caché est envoyé ; le délai (15 s ?).

## Mises à jour de documents à prévoir

- **Guide** (`GUIDE.md`) : corriger `:816`, `:1692`, `:1801` ; après construction, une partie « Vérifier un formulaire » et « Envoyer une commande ».
- **Leçons** : une leçon par notion, dans `exemples/lecons/` : « Un champ obligatoire et un courriel », « Annoncer un message », « Joindre des valeurs à un envoi », « Un champ selon un choix ». Chacune avec son `.test`.
- **`NOMS.md`** : retirer `form, input, textarea, select, label` de « Pas encore là » (`:228`) ; ajouter les mots retenus, face à HTML et Flutter.
- **`COMPARAISON-WEB.md`** : corriger `:3`, `:81`, `:84`, `:99`, `:160`, `:187`.
- **`TABLEAU-WEB.md`** : `Form` à environ 60 % tant que la vérification manque (`:518`) ; le courriel sans attendre les comptes, si A1 est retenue (`:524`) ; puis remonter après construction ; republier la page en ligne du tableau à chaque changement du langage.
- **Site de référence** : la réservation dans `panier.holo` ; passer X01 de `BLOQUÉ` à une vraie recette.
- **Journal** et **`AGENTS.md`** : l'entrée de l'exploration (fait par Claude, sans l'annoncer).

Relu le 2026-10-07 : 14 corrections.
