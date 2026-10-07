# ADR-068 — Des formulaires qui vérifient

- Statut : PROPOSITION (construit et essayé ; attend la validation de Yocthan)
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07, lot 3 (« champs obligatoires, e-mail, longueurs, messages d'erreur accessibles, touche Entrée, envoi unique et délai maximal ») ; `ADR-042` (le formulaire `Form`, `send`, `sent`, `failed`) ; l'exploration #82 (piste 2).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Un formulaire envoyait ce que contenaient ses champs, sans rien vérifier :
- pas de champ obligatoire, ni d'adresse e-mail, ni de longueur minimale ;
- aucun message ;
- la touche Entrée n'envoyait rien ;
- un serveur qui ne répondait pas faisait attendre sans fin ;
- le serveur rangeait n'importe quel message, même forgé sans passer par la page.

## Décision

1. **Obligatoire** : `required: true` sur un `Input` qui écrit un texte (aussi `type: email`, `date`, `time`, `color`, `file`), une `Checkbox` (cochée) ou un `Choice` (une réponse choisie).
   - Un nombre n'est jamais vide : `required` y est refusé, avec la raison ; on le borne, `min: 1`.
   - Un champ obligatoire hors d'un `Form` est refusé : c'est l'envoi qui vérifie.
2. **E-mail** : `Input(type: email)`. Le navigateur montre le clavier des adresses et propose celle du visiteur. Le moteur vérifie une adresse plausible : une arobase, un point dans le domaine, pas d'espace, 254 caractères au plus.
3. **Longueurs** : `min:` et `max:` sur un champ de texte, en caractères (`max:` existait déjà).
4. **Les messages** :
   - à l'envoi (`Contact.send`), le moteur vérifie chaque champ ;
   - ce qui ne va pas s'écrit sous le champ (`aria-describedby`), et le champ est marqué (`aria-invalid`) ;
   - le premier champ à corriger reçoit le clavier, et un lecteur d'écran annonce le message, ou le nombre de champs à corriger ;
   - les messages suivent ce qu'on corrige ;
   - ils sont écrits par le moteur, dans la langue de la page (français ou anglais), par exemple « Ce champ est obligatoire. », « Écris une adresse e-mail, comme nom@exemple.fr. », « Au moins 2 caractères. ».
5. **Entrée** dans un champ d'une ligne envoie le formulaire : le premier bouton du formulaire est touché, comme sur le web.
6. **Un seul envoi à la fois** (c'était déjà le cas), et le formulaire le dit (`aria-busy`) pendant l'envoi.
7. **15 secondes au plus** : un serveur qui ne répond pas est un échec (`Contact.failed`).
8. **Le serveur vérifie à nouveau**, par le moteur (`holo form page.holo`, le message sur l'entrée standard) :
   - les mêmes règles ;
   - aucun champ inconnu ;
   - chaque valeur de sa sorte (texte ou nombre).

   Un message refusé reçoit 422 et la liste des erreurs. La page peut être contournée ; le serveur, non.
9. **En chemin** : un nombre à virgule part comme on l'écrit, « 12.50 », et non plus à son échelle interne, « 1250 » (défaut de l'`ADR-066`).

## Comparaison faite avant de choisir

| Option | Écriture | Pour | Contre |
|---|---|---|---|
| **A. Les mots du web, vérifiés par le moteur** (proposée) | `required: true`, `type: email`, `min: 2` | les mots que connaissent ceux qui ont vu le web ; un seul vérificateur, pour la page et le serveur | les messages sont ceux du moteur (français ou anglais) |
| B. Laisser faire le navigateur | `required` dans le HTML | rien à écrire | des bulles différentes d'un navigateur à l'autre, mal lues par les lecteurs d'écran, en anglais selon le navigateur ; le serveur n'est pas protégé |
| C. Des règles écrites par l'auteur | `If(email, not: …)` | souple | chaque auteur réinvente ; facile à oublier côté serveur |

Défauts du web évités :

- la validation du navigateur, qui ne se style pas et se lit mal ;
- une vérification faite seulement dans la page ;
- `type=email` qui accepte « a@b » ;
- un `fetch` sans délai ;
- un double clic qui envoie deux fois.

## Conséquences

- Les messages ne se personnalisent pas encore champ par champ : à faire si le besoin vient (un `error:`).
- Le serveur de démonstration vérifie quand le moteur en ligne de commande est construit. Sans lui, il range le message comme avant.

## Critères de validation

- Test du moteur `a_form_is_checked_before_sending_and_again_by_the_server` :
  - les quatre messages, la langue anglaise, l'adresse e-mail ;
  - le serveur, avec un bon message, un message forgé (5 erreurs), illisible, ou d'un autre formulaire ;
  - le nombre à virgule envoyé « 12.50 » ;
  - trois refus.
- Dans Chrome, la leçon 88 :
  - vide, 4 messages, le clavier sur le premier champ, rien d'envoyé ;
  - « A » donne « Au moins 2 caractères. », « ada@ » donne le message de l'e-mail, et tout corrigé, plus de message ;
  - Entrée envoie, une seule fois ;
  - un serveur muet donne un échec après 15 secondes ;
  - un message forgé reçoit 422.
