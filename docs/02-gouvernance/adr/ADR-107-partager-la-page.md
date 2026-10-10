# ADR-107 — Partager la page : `Device(kind: share)`

- Statut : ACCEPTÉ (Yocthan, 2026-10-09 : « tu le valides déjà, tu le fais déjà »)
- Date : 2026-10-09
- Responsable : Yocthan Mabeka
- Discussions sources : l'issue #236 (« Dernière dette du web : le partage du téléphone »), l'une des douze dernières dettes du web, validées d'avance par Yocthan le 2026-10-09 ; le grand tableau du web, où « presse-papiers, partage » était « en partie » (`Device(kind: clipboard)` et `write` ne faisaient que copier) ; `ADR-094` (l'appareil sur permission), que cette décision complète.
- Validation : Yocthan, le 2026-10-09, d'avance, avec les douze dernières dettes du web.
- Projets affectés : HoloCode, HoloEngine

## Contexte

- Sur un téléphone, partager une page est un geste de tous les jours : le bouton « Partager » ouvre une feuille où l'on choisit à qui l'envoyer, par un message, un e-mail. Le web l'offre par `navigator.share` (Web Share).
- HoloCode savait copier un texte de la page dans le presse-papiers (`Device(kind: clipboard)`, `write`), pas ouvrir cette feuille.
- Écrit à la main, le partage du web a trois pièges :
  - `navigator.share` n'existe pas partout (Chrome sous Linux, Firefox sur un ordinateur). Une page qui l'appelle sans vérifier casse ; une page qui vérifie cache souvent son bouton, et l'ordinateur perd le geste.
  - Il ne marche que pendant le geste du visiteur : un appel fait après une attente (une réponse du serveur, un minuteur) est refusé par le navigateur.
  - Le visiteur qui ferme la feuille sans rien choisir fait échouer l'appel (`AbortError`), et beaucoup de pages affichent alors une erreur.

## Décision

```holo
Page(
  title: "The Saturday market",
  state: State(shared: 0),
  children: [
    Device(name: Share, kind: share, label: "Sharing this page"),
    Button(name: ShareIt, text: "Share this page"),
    P("Shared {shared} times during this visit."),
  ],
  rules: [ On(ShareIt.tap, effect: Share.request), On(Share.done, effect: shared.add(1)) ],
)
```

1. **`Device(kind: share)`**, une cinquième sorte d'appareil : `Share.request` partage la page.
   - Sur un téléphone, la feuille de partage s'ouvre avec le titre de la page (celui de l'onglet, `Page(title:)`) et son adresse, telle que la barre d'adresse la montre, avec ses valeurs (`ADR-091` : le lien partagé ouvre le même onglet).
   - Sans feuille de partage (un ordinateur), le même bouton copie l'adresse dans le presse-papiers.
   - La page dit ce qui s'est passé, à l'écran et au lecteur d'écran, dans la zone d'état du bloc (`role="status"`, `aria-live="polite"`, la même que pour les autres sortes) : « Page partagée. », « Adresse de la page copiée : colle-la où tu veux. », « Partage annulé. », ou la panne avec l'adresse écrite en entier, à copier à la main.
2. **Dans le toucher même** : le partage part du toucher d'un bouton (`On(Bouton.tap, …)`, la règle d'`ADR-094`), et le moteur appelle le navigateur pendant le clic, avant toute attente : rien ne se glisse entre le geste et la feuille. Jamais d'une minuterie, d'une règle qui guette, ni d'une fin (`Share.done`).
3. **Ce que la page apprend** :
   - `Share.done` : la page est partagée (le visiteur a choisi où l'envoyer), ou son adresse est copiée ;
   - `Share.failed` : rien n'a marché (ni feuille ni presse-papiers, ou un refus du navigateur) ; l'adresse est alors écrite dans la zone d'état ;
   - **fermer la feuille sans rien choisir** (`AbortError`) n'est ni l'un ni l'autre : c'est le choix du visiteur, pas une panne. La zone d'état dit « Partage annulé. », et la page n'a rien à faire.
4. **Sans JavaScript**, le bloc montre son étiquette et une phrase : « Sans JavaScript, ce bouton ne partage pas : copie l'adresse de la page dans la barre du navigateur, ou prends « Partager » dans son menu. » Le reste de la page reste lisible.
5. Le partage ne rend rien à la page : `value:` est refusé. On ne lui demande que `request`, et `stop`, permis partout comme pour les autres sortes : il oublie un partage en cours.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où l'écrire | un bloc `Share` ; un réglage du bouton, `Button(share: true)` ; une capacité de la page, `Shop.share` ; **une sorte de plus pour `Device`** | `ADR-094` : « une sorte de plus ne fera pas un mot de plus ». `Device` a déjà le toucher d'un bouton exigé, la zone d'état lue par le lecteur d'écran, `done` et `failed`. Un réglage du bouton mêlerait ce qu'est le bouton et ce que fait son toucher, que disent les règles (`ADR-015`) |
| Le nom de la sorte | `link` ; `send` ; **`share`** | le mot du web (`navigator.share`) et celui du bouton du téléphone (« Partager »). `Shared` (les valeurs partagées entre visiteurs, `ADR-079`) est un bloc, `share` une valeur de `kind:` : ils ne s'écrivent pas au même endroit |
| L'action | `Share.share` ; `Share.open` ; **`Share.request`** | rien de nouveau : `request` est déjà « demander au navigateur » pour chaque sorte d'appareil ; ici, la feuille de partage |
| Ce qui est partagé | un texte choisi par l'auteur (`text:`) ; **le titre et l'adresse de la page** | ce que fait le bouton « Partager » du navigateur ; un texte à soi attendra (dette) |
| Sans feuille de partage | rien ; un bouton caché sur l'ordinateur ; **l'adresse copiée** | la règle de parité de Yocthan : ce qui existe sur le téléphone existe sur l'ordinateur. Le même bouton marche partout, et la page dit ce qu'elle a fait |
| La feuille fermée sans rien choisir | `failed` ; `done` ; **ni l'un ni l'autre** | `failed` ferait afficher une panne à tort, `done` un merci à tort ; la zone d'état le dit au visiteur |
| Sans JavaScript | rien ; un lien `mailto:` ; **une phrase qui dit comment partager quand même** | la page fabriquée ne connaît pas son adresse entière (le nom du serveur) ; le navigateur a sa barre d'adresse et son menu « Partager » |

## Ce qui est refusé, et pourquoi

- `Share.request` hors du toucher d'un bouton (une minuterie, une règle qui guette, `Share.done`) : le navigateur refuserait, et une page ne partage pas sans qu'on le lui demande.
- `Share.write`, et toute autre action que `request` et `stop` : un partage s'ouvre, il ne s'écrit pas.
- `value:` : le partage ne rend rien à la page.

## Les défauts du web évités

- **Le bouton qui casse ou disparaît sur un ordinateur** (`navigator.share` absent) : ici, il copie l'adresse.
- **Le partage refusé parce qu'il part après une attente** : ici, l'appel part dans le clic même.
- **La feuille fermée prise pour une panne** (`AbortError`) : ici, « Partage annulé. », sans signal.
- **Un partage silencieux** : ici, la page dit ce qui s'est passé, à l'écran et au lecteur d'écran ; en cas de panne, l'adresse est écrite.

## Dettes

- Un texte choisi par l'auteur (`text:`), une autre adresse que celle de la page, un fichier (le web le permet sur certains téléphones).
- Un vrai téléphone : la feuille de partage d'Android et de l'iPhone, avec TalkBack. L'essai remplace `navigator.share`.
- Le grand tableau du web : « presse-papiers, partage » passe de « En partie » à « Oui », à la fusion.

## Critères de validation

- Test du moteur : `a_page_is_shared_from_a_button_and_nothing_else` (accepté sur un bouton ; refusé d'une minuterie, d'une règle qui guette et de `Share.done` ; `Share.write` et `value:` refusés ; la zone d'état et la phrase sans JavaScript).
- Dans Chrome : « partager la page : la feuille du téléphone avec le titre et l'adresse, sinon l'adresse copiée (leçon 130) ».
  - Un ordinateur sans partage : l'adresse copiée (relue dans le presse-papiers), dite dans la zone `aria-live`.
  - Un téléphone (`navigator.share` remplacé avant la page) : le titre et l'adresse reçus, pendant le clic même.
  - La feuille fermée : « Partage annulé. », ni `done` ni `failed`.
  - Une panne : l'adresse écrite, `failed`.
  - axe-core : zéro défaut.
- Leçon `130-partager-la-page.holo`.
