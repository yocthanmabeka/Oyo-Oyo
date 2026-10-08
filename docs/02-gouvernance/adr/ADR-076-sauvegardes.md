# ADR-076 — Lot 5, troisième pas : les sauvegardes de la base

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (lot 5 : « `holo serve`, SQLite, sauvegardes, adresses ») ; `ADR-074`, `ADR-075`
- Validation : Yocthan, le 2026-10-07 : « Oui, vas-y et continue sur le lot 5 »
- Projets affectés : le serveur

## Décision

1. **`holo serve` sauvegarde sa base** au démarrage si la dernière copie a plus d'un jour, puis une fois par jour tant qu'il tourne.
2. **`holo backup [dossier]`** fait une sauvegarde tout de suite, et écrit son chemin.
3. Une sauvegarde est **une copie entière et cohérente** de `holo-data/site.sqlite`, même pendant que le serveur écrit (`VACUUM INTO` de SQLite), dans `holo-data/backups/site-2026-10-07-2310-30.sqlite` (l'heure universelle). Elle se relit seule, avec n'importe quel outil SQLite.
4. **Les quatorze plus récentes sont gardées** ; les plus anciennes sont effacées.
5. **Revenir à une sauvegarde** : arrêter `holo serve`, copier la sauvegarde à la place de `holo-data/site.sqlite`, relancer. Rien n'est automatique : on ne remplace pas une base sans que l'auteur le décide.

## Ce qui n'est pas fait

- Une copie ailleurs que sur le même disque (une clé USB, un autre PC) : c'est à l'auteur de la faire ; aucun service extérieur n'est appelé (chez soi d'abord).
- Les fichiers reçus (`holo-data/files/`) ne sont pas dans la copie de la base : ils sont déjà des fichiers, à copier avec le dossier.

## Critères de validation

- Tests du moteur : la copie se relit et contient la valeur d'un visiteur ; avec vingt anciennes copies, il en reste quatorze, les plus récentes ; une copie n'est jamais servie ; un dossier jamais servi n'a rien à sauvegarder.
- À la main : `holo serve` écrit « Sauvegarde : … » au démarrage ; `holo backup` en ajoute une.
