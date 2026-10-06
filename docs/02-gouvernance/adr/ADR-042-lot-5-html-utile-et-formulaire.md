# ADR-042 — Lot 5 : le HTML utile, et l'envoi d'un formulaire

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : le grand tableau (`docs/01-holocode/TABLEAU-WEB.md`) ; la question « où vont les messages ? », tranchée par Yocthan le 2026-10-06 en suivant la recommandation de Claude : dans un fichier de son serveur local, qu'il lit lui-même.
- Validation : validé par Yocthan le 2026-10-06, après avoir tout essayé : « En fait, j'ai tout testé de tout ce qui était à laisser [à l'essai] et je trouve que c'est bon. Donc, euh, valide-le. »
- Projets affectés : HoloCode, HoloEngine, le serveur d'essai

## Contexte

Il restait au HTML deux urgences (envoyer un formulaire ; `fetch` pour envoyer) et une douzaine d'éléments utiles : l'icône de l'onglet, le barré et le surligné, l'exposant et l'indice, le lien vers un endroit de la page, l'image plus légère sur téléphone, la légende d'une image, le lecteur de son, la glissière, les champs de date, d'heure et de couleur, la barre de progression, le pli qui s'ouvre, la fenêtre par-dessus.

## Décision (à l'essai)

1. **`Page(icon: "etoile.svg")`** : l'icône de l'onglet (`.png`, `.svg`, `.ico`).
2. **Dans un texte** : `~~barré~~`, `==surligné==`, `m^2^` (exposant), `H~2~O` (indice). Le souligné n'est pas offert : il ressemble à un lien.
3. **`A("…", to: "#Horaires")`** mène au bloc nommé `Horaires` : il reçoit cet `id`, et le navigateur y descend sans le moteur. Un nom qui n'existe pas est refusé.
4. **`Image(…, caption: "…", phone: "petite.jpg")`** : une légende (`figure`, `figcaption`) ; une image plus légère pour un écran de téléphone (`picture`), seule téléchargée.
5. **`Sound(source:, label: "…")`** : avec une étiquette, un son devient un lecteur, avec ses boutons, jamais lancé seul.
6. **`Slider(value:, label:, min:, max:)`** : une glissière ; la valeur reste dans ses bornes.
7. **`Input(…, type: date | time | color)`** : la valeur est un texte, et n'accepte que ce format (`2026-10-06`, `14:30`, `#e9b44c`).
8. **`Progress(value:, max:, label:)`** : une barre qui suit un nombre de la page.
9. **`Details(summary:, children:, open:)`** : un pli qui s'ouvre ; il marche sans le moteur.
10. **`Dialog(name:, children:)`** : une fenêtre par-dessus la page ; `Confirm.open`, `Confirm.close` ; la croix et la touche Échap la ferment aussi.
11. **`Form(name:, children:)`** et **`Contact.send`** : le formulaire envoie, au serveur d'où vient la page, les valeurs que présentent ses champs ; il dit ensuite **`Contact.sent`** ou **`Contact.failed`**. Le serveur d'essai range chaque message dans `messages/` à la racine du dépôt (un fichier par page, une ligne par message, 16 Ko au plus par message) ; ce dossier n'est jamais versionné. Rien ne part ailleurs. Une page ne recharge jamais en envoyant.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où vont les messages | un fichier sur le serveur local ; un e-mail (un service, un compte) ; attendre un serveur en ligne | **Un fichier local** : décidé par Yocthan sur recommandation ; rien à louer, tout reste chez lui. |
| Comment envoyer | un `<form>` qui recharge la page ; **une règle qui envoie, puis un signal** | Une règle : l'auteur dit quoi faire après (merci, erreur), et la page ne se recharge jamais. Ce qui part est seulement ce que les champs du formulaire présentent. |
| La fenêtre | une valeur et un `If` posé par-dessus ; **`Dialog`** | `Dialog` : la vraie fenêtre du navigateur, qui garde le clavier dedans et se ferme à Échap ; un `If` ne sait pas faire cela. |
| Le pli | une valeur et un `If` ; **`Details`** | `Details` : il marche sans le moteur, et un lecteur d'écran le comprend. |
| La glissière | `Input(type: range)` ; **`Slider`** | `Slider` : le mot de Flutter, et une glissière n'est pas un champ où l'on écrit. |
| Date, heure, couleur | trois blocs ; **`Input(type:)`** | C'est un champ, avec le choisisseur du navigateur ; la valeur est un texte contrôlé. |
| L'image pour téléphone | `srcset` et ses largeurs ; **`phone:`** | Le même seuil que l'état `phone:` des styles (`ADR-041`) ; une seule image de plus. |
| Le souligné | `__…__` ; **rien** | Sur le web, ce qui est souligné ressemble à un lien. |
| Défauts du web évités | une ancre qui ne mène nulle part ; un formulaire qui recharge la page et perd ce qu'on a écrit ; un lecteur qui démarre seul | refusée ; jamais ; jamais. |

## Conséquences

- Restent pour plus tard (classés ainsi dans le tableau) : `aside`, `abbr`, `time`, `address`, les listes de définitions, les champs e-mail, mot de passe et fichier, `fieldset`, `datalist`.
- Un serveur en ligne recevra un jour ces messages autrement : la page n'aura pas à changer, seulement le serveur.

## Critères de validation

- Leçons 55 à 65 ; test du moteur `plat.rs` (`le_lot_5_le_html_utile_et_le_formulaire`) ; dans Chrome : chaque élément, et un vrai message arrivé dans `messages/`.
