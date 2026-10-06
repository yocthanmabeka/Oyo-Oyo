# ADR-039 — Lot 2 : le survol qui agit, le « sinon », l'attente unique, l'heure du visiteur

- Statut : ACCEPTÉ
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : le grand tableau (`docs/01-holocode/TABLEAU-WEB.md`) ; Yocthan, le 2026-10-06 : « tu travailles sur le lot 2 jusqu'au lot 5… je suis tes recommandations »
- Validation : validé par Yocthan le 2026-10-06, après avoir tout essayé : « En fait, j'ai tout testé de tout ce qui était à laisser [à l'essai] et je trouve que c'est bon. Donc, euh, valide-le. »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Le lot 2 prend quatre manques que le tableau classe « utiles » ou « en priorité » : le survol comme signal, le « sinon », « une seule fois, plus tard » (`setTimeout`) et la date du jour.

## Décision

1. **`On(Carte.hover, effect: …)` et `On(Carte.hoverEnd, effect: …)`** : tout bloc nommé qui se voit émet ces deux signaux. Un survol change des valeurs ou joue un son ; il n'emmène jamais ailleurs (`enter`, `portals` demandent un toucher). Il est atteignable partout : à la souris, au clavier (le bloc reçoit le focus avec Tab), au doigt (toucher survole, toucher ailleurs quitte). La page légère fait venir le moteur au premier survol et le rejoue s'il dure encore.
2. **`If(…, children: [ … ], else: [ … ])`** : ce qu'on montre quand la condition est fausse.
3. **`After(3s, effect: …)`** : une seule fois, après la durée (100ms à 3600s). Dans les règles de la page, l'attente part à l'ouverture. Sous une condition, `If(message, is: 1, rules: [ After(3s, …) ])`, elle part quand la condition devient vraie et s'arrête si elle redevient fausse : le message qui s'efface tout seul.
4. **L'heure du visiteur** : six valeurs que le moteur donne, comme `count` et `total` : `year`, `month` (1 à 12), `day` (1 à 31), `weekday` (1 lundi … 7 dimanche), `hour` (0 à 23), `minute`. On les montre et on les compare ; on ne les change pas, on ne les garde pas, et ces noms ne peuvent plus servir à ses propres valeurs. La page lit l'heure de l'appareil et se tient à jour à chaque minute ; `When(hour, is: 12, …)` sonne au changement d'heure. Le serveur fabrique la page d'avance avec son heure ; le moteur la corrige à son arrivée.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le nom du survol | `near` (proposé en 2026-10 pour la profondeur) ; **`hover`** | `hover` : le style dit déjà `hover:` (`ADR-036`), un seul mot pour une seule idée. En profondeur, le même signal servira quand le personnage approche. |
| La fin du survol | un seul signal qui s'annule seul ; **deux signaux** | Deux signaux : rien de caché ; l'auteur écrit ce qui arrive au départ. |
| Le survol sur un téléphone | rien (comme le web : le survol y est perdu) ; **toucher survole** | Le web cache une information au survol qu'un téléphone ne peut jamais voir ; ici, le doigt l'atteint. |
| Le survol au clavier | rien (comme `mouseenter`) ; **focus avec Tab** | Le défaut du web : ce qui n'apparaît qu'à la souris est perdu pour qui navigue au clavier. |
| Le « sinon » | deux `If` contraires ; un bloc `Else` à côté ; **`else:` dans le même `If`** | `else:` : la condition est écrite une fois, et le « sinon » ne peut pas se perdre loin de son `If`. |
| Une fois, plus tard | `Every` qui se coupe lui-même ; **`After`** | `After` : un mot court ; le départ sous condition remplace `clearTimeout`, qu'on oublie en JavaScript. |
| L'heure | un bloc `Clock` à déclarer ; des noms préfixés (`nowHour`) ; **six noms simples, réservés** | Six noms simples : `{hour}` se lit tout seul. Le prix : ces noms ne servent plus à ses propres valeurs ; le moteur le dit avec le bon message. |

## Conséquences

- L'heure rend la page « vivante » : le moteur arrive tout de suite pour la tenir à jour (`ADR-033`).
- `{minute}` s'affiche sans zéro devant (`14 h 5`) : les formats de date et de nombre (`Intl`) restent à faire. Un compte à rebours jusqu'à une date demande un calcul sur les dates : à faire.
- Restent dans les lots suivants : les listes et la répétition (lot 3), le CSS utile (lot 4), le HTML utile et l'envoi d'un formulaire (lot 5).

## Critères de validation

- Leçons 45 à 48 ; test du moteur `plat.rs` (`le_lot_2_survol_sinon_attente_heure`) ; essais dans Chrome à la souris, au clavier et au doigt, et passage de l'heure à la minute suivante.
