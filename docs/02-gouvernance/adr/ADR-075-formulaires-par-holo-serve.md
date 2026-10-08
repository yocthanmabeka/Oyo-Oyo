# ADR-075 — Lot 5, deuxième pas : les formulaires `Form` reçus par `holo serve`, avec ou sans JavaScript

- Statut : ACCEPTÉ
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (lot 5, et la condition « la boutique marche avec JavaScript coupé : ajouter, retirer, commander ») ; `ADR-042` (les formulaires), `ADR-059` (les fichiers), `ADR-068` (les formulaires qui vérifient), `ADR-074` (`holo serve`)
- Validation : Yocthan, le 2026-10-07 : « Oui, vas-y et continue sur le lot 5 » ; vérifié dans Chrome, avec et sans JavaScript, avant la fusion
- Projets affectés : HoloEngine, le serveur

## Décision

1. **`holo serve` reçoit les formulaires** que le moteur du navigateur envoie, en JSON, avec ou sans fichiers, comme le faisait `outils/server.mjs` : le moteur vérifie à nouveau chaque champ (`ADR-068`), les fichiers sont reconnus à leurs premiers octets et rangés sous un nom tiré au hasard (`ADR-059`). Le message va dans la base, `holo-data/site.sqlite` (table `messages`), les fichiers dans `holo-data/files/<page>/`. Rien de cela n'est servi.
2. **Sans JavaScript, un `Form` s'envoie aussi.** Ses champs et son bouton se rattachent au formulaire des gestes, comme les autres (`ADR-074`). Quand le toucher demande l'envoi (`On(Send.tap, effect: Contact.send)`), le serveur vérifie les champs avec le même code que le navigateur :
   - s'il manque quelque chose, rien n'est rangé, et la page revient avec les messages sous les champs, reliés à eux (`aria-describedby`, `aria-invalid`), exactement comme le moteur les écrit ; ils suivent ensuite ce que le visiteur corrige ;
   - sinon, le message est rangé, puis `Contact.sent` passe par l'arbitre : la page dit « merci ».
3. **Avec JavaScript, la page reprend ses gestes ordinaires** dès qu'elle arrive : les boutons et les champs quittent le formulaire des gestes, et tout se passe comme avec `outils/server.mjs` (la touche Entrée dans un `Form` l'envoie, la page ne se recharge pas).
4. **`holo messages [dossier]`** montre à l'auteur les messages reçus, une ligne JSON chacun : quand, quelle page, quel formulaire, les valeurs, les fichiers.
5. **Les limites** : 16 Ko pour un message, 40 Mo avec ses fichiers, 5 Mo de messages et 500 Mo de fichiers par page.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Où ranger les messages | un fichier `.jsonl` par page, comme `outils/server.mjs` ; **la base SQLite** | une seule base pour tout le site : les visites, les messages, demain les comptes ; une sauvegarde d'un seul fichier |
| Les messages d'erreur sans JavaScript | une page d'erreur à part ; **la même page, les messages sous les champs** | le visiteur garde ce qu'il a écrit, et lit les mêmes phrases qu'avec le moteur |
| La touche Entrée sans JavaScript dans un `Form` | envoyer ; **garder les champs sans envoyer** | un seul formulaire des gestes par page : Entrée ne sait pas quel `Form` envoyer ; le bouton « Envoyer » le sait |

## Ce qui n'est pas fait

- Sans JavaScript, un fichier ne part pas (`Input(type: file)` reste au moteur).
- Les sauvegardes de la base et les adresses `profil/{id}.holo` : la suite du lot 5.
- L'éditeur et la pile restent dans `outils/server.mjs`.

## Critères de validation

- Tests du moteur : les messages sous les champs (un champ, un groupe de boutons ronds, un texte avec `<`) ; sans JavaScript, envoyer vide garde les messages et ne range rien, corrigé range et passe `Contact.sent` ; envoyé par le moteur : `204`, `422` pour un champ trop court, un champ inconnu ou un message illisible ; une image reconnue à ses octets et rangée, un faux PNG refusé (`415`), le fichier rangé jamais servi.
- Dans Chrome (`node outils/browser-tests.mjs serve`, leçon 88) : sans JavaScript, quatre messages reliés à leurs champs, puis « Merci » ; avec JavaScript, un nouveau visiteur, « Merci » sans recharger ; deux messages lus par `holo messages`.
