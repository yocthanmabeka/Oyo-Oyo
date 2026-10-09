# ADR-089 — Les secondes, et un chronomètre

- Statut : ACCEPTÉ
- Date : 2026-10-08
- Responsable : Yocthan Mabeka
- Discussions sources : la demande de Yocthan, sur son téléphone, devant la leçon 48 (des secondes, et un chronomètre au centième) ; la tâche confiée à la session du nuage le 2026-10-08 (tableau « Qui fait quoi », PR 175 de la session du PC) : « `{second}`, donnée chaque seconde seulement si la page l'affiche ; les millisecondes par un bloc chronomètre (démarrer, arrêter, remettre à zéro) que la page dessine au rythme de l'écran, le moteur ne recevant que le temps final »
- Validation : Yocthan, le 2026-10-08, en confiant la tâche à la session du nuage ; vérifié dans Chrome avant la fusion
- Projets affectés : HoloCode, HoloEngine

## Décision

1. **`second`** rejoint l'heure du visiteur (`ADR-039`) : de 0 à 59, on la lit (`{second}`), on la compare, on ne la change pas, et elle ne peut pas servir de nom à ses propres valeurs.
2. **Une page qui affiche la seconde la reçoit chaque seconde ; elle seulement.** Les autres pages continuent de recevoir l'heure à chaque minute : rien ne tourne pour rien.
3. **`Stopwatch(name: Chrono, value: time, label: "…")`** : un chronomètre au centième de seconde. Trois demandes : `Chrono.start`, `Chrono.stop`, `Chrono.reset`.
   - La page le fait tourner elle-même, au rythme de l'écran, d'après l'horloge de l'appareil : juste, même si l'onglet a été caché un moment.
   - Le moteur ne reçoit que **le temps final**, en millisecondes, dans la valeur `value` ; les règles qui la guettent répondent (`When(time, under: best, …)`), puis le signal `Chrono.stopped`.
   - Démarrer à nouveau continue le compte ; remettre à zéro efface le cadran, et la valeur garde le dernier temps final.
   - Le lecteur d'écran ne l'annonce pas à chaque centième (`role="timer"`) ; le temps final est annoncé à l'arrêt, avec son `label`.
   - Les chiffres ont tous la même largeur : le cadran ne tremble pas.
4. **`{time:stopwatch}`** écrit un temps de chronomètre : `01:23,45`, et `1:02:03,45` quand il y a des heures ; la virgule ou le point selon la langue de la page.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Les centièmes | les donner au moteur à chaque image ; **la page les dessine, le moteur reçoit le temps final** | le moteur ne travaille pas 60 fois par seconde ; un chronomètre reste juste et léger |
| La seconde | toutes les pages, chaque seconde ; **seulement celles qui l'affichent** | une page qui montre l'heure à la minute ne réveille pas l'appareil chaque seconde (la batterie) |
| Remettre à zéro | effacer aussi la valeur ; **effacer le cadran seulement** | la valeur garde le dernier temps : le meilleur temps ne se perd pas |

## Ce qui n'est pas fait

- Des tours (des temps intermédiaires), un compte à rebours.
- La seconde dans la page fabriquée d'avance par le serveur : elle vaut 0 jusqu'à l'arrivée du moteur, qui vient tout de suite sur une page qui lit l'heure.

## Critères de validation

- Tests du moteur : la seconde donnée et lue, seulement par une page qui l'affiche ; `second` refusé comme nom de valeur ; le format `stopwatch` (minutes, heures, langue) ; le chronomètre dessiné (`role="timer"`, son `label`, le temps gardé) ; le temps final reçu, puis `stopped` ; le meilleur temps gardé par des règles qui guettent ; refusés : sans nom, une valeur absente, un nombre à virgule, un `label` qui n'est pas un texte, un réglage inconnu, une demande inconnue.
- Dans Chrome (leçons 48 et 111) : la seconde change ; le cadran au repos `00:00,00`, en marche, puis fixe une fois arrêté ; le temps écrit dessous ; remis à zéro.
