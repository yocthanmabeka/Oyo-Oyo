# Proposition : un serveur en Rust qui fait tourner le même arbitre, puis des comptes

- Auteur : Claude, d'après une question de Yocthan
- Date : 2026-10-07
- **Statut : proposition, à discuter avec Yocthan.** Rien n'est construit. Yocthan, le 2026-10-07 : « on veut un langage qui construit le métaverse de A à Z […] fluide, limpide ».

## La question de Yocthan

> Les langages issus de JavaScript n'utilisent-ils pas vraiment Node.js pour gérer le serveur ou les comptes ?

**Oui.** Dans le monde JavaScript, la page tourne dans le navigateur, et le serveur tourne presque toujours dans Node.js (avec Express, Next.js, SvelteKit, Nuxt), ou chez un service tout fait (Firebase, Supabase). L'auteur écrit donc deux programmes : celui de la page et celui du serveur. Avec TypeScript des deux côtés, c'est le même langage, mais ce sont deux programmes qu'il faut garder d'accord. C'est là que naissent beaucoup de failles : la page vérifie une chose, le serveur en oublie une autre.

HoloCode a aujourd'hui un petit serveur d'essai en Node.js (`moteur/outils/server.mjs`) : il sert les pages, range les messages et les fichiers. Il est fait pour le PC de Yocthan, pas pour Internet.

## L'idée : un seul moteur, des deux côtés

Le cœur de HoloCode est déjà un **arbitre pur** : un état et un signal donnent un nouvel état. Il est écrit en Rust et tourne dans le navigateur (compilé en WebAssembly). Le même code peut tourner sur un serveur, sans rien réécrire.

```
          navigateur                                serveur (Rust)
   page.holo ─► arbitre ─► HTML          page.holo ─► arbitre ─► ce qui est gardé
                  │                                     ▲
                  └──── « Like.tap » (un signal) ───────┘
```

- **L'auteur n'écrit pas de serveur.** Il écrit sa page ; il dit, en HoloCode, ce qui est gardé et partagé, et qui peut le voir.
- **La page envoie des gestes, pas des valeurs.** Le serveur rejoue le geste avec le même arbitre, sur sa propre copie : il ne croit jamais ce que dit le navigateur. C'est déjà ce que fait l'envoi d'un fichier (`ADR-059`) : le serveur demande au moteur ce que la page permet.
- **Un seul binaire** : `holo serve dossier/` sert le site, fabrique les pages, reçoit les formulaires et les fichiers, garde les données.

Pour la fluidité que veut Yocthan (« comme si on faisait lire deux films », ce qu'on ne veut pas) : il n'y a qu'un langage, qu'un moteur, qu'une vérité. La page et le serveur ne peuvent pas diverger, puisque c'est le même arbitre.

## Par étapes

| Étape | Ce qu'on gagne | Écriture proposée (à discuter) | Séances |
|---|---|---|---|
| **1. `holo serve`** | Le serveur d'essai devient un vrai serveur en Rust : pages, formulaires, fichiers, HTTPS derrière un relais | rien de nouveau pour l'auteur | 2 |
| **2. Des valeurs partagées** | Un compteur de visites, un livre d'or, des « j'aime » vus par tous | `State(likes: 0, shared: [likes])` ; le serveur garde `likes` et arbitre les gestes de tous | 2 à 3 |
| **3. Des comptes** | Chacun a ses valeurs : son panier, ses tableaux, sa progression | `Page(account: optional)` ; `{account.name}` ; `If(account, …)` ; `State(cart: 0, mine: [cart])` | 3 à 4 |
| **4. En direct, à plusieurs** | Voir les autres bouger : le pont vers le métaverse à plusieurs (`PLAN-3D.md`) | les valeurs partagées arrivent d'elles-mêmes, sans recharger | 3 |

Chaque étape aurait sa leçon, son ADR, ses tests, comme d'habitude.

## Les comptes, sans mot de passe

Garder des mots de passe, c'est garder ce que les pirates cherchent. Deux façons modernes de s'en passer :

- **Les clés d'accès (passkeys)** : le téléphone ou le PC prouve qui on est, avec l'empreinte ou le visage. Rien de secret n'est gardé sur le serveur. Google, Apple et Microsoft les poussent depuis 2023.
- **Un lien par e-mail** : on reçoit un lien qui ouvre la session. Il faut un service d'envoi d'e-mails (un compte et une clé secrète, chez Resend ou Postmark par exemple).

Je propose les **clés d'accès d'abord**, et le lien par e-mail en secours pour un appareil qui ne les connaît pas.

## Où l'héberger

- **Les données** : un seul fichier SQLite à côté du site, sauvegardé chaque nuit. Assez pour des milliers de visiteurs.
- **La machine** : un petit serveur loué (environ 5 € par mois, en Europe pour le RGPD), ou un service comme Fly.io ; le PC de Yocthan pour les essais.
- **HTTPS** : un relais comme Caddy, qui obtient le certificat tout seul.

## Sécurité, ce que le moteur garantirait

- Aucun code libre, côté page comme côté serveur : seulement ce que le moteur sait faire, borné.
- Le serveur rejoue chaque geste avec l'arbitre ; une valeur envoyée par le navigateur n'est jamais crue.
- Des limites partout : taille des envois, nombre de gestes par minute et par visiteur, place gardée par page et par compte.
- Les envois ne sont acceptés que depuis le site lui-même (même origine).
- Un compte peut être effacé, avec tout ce qu'il a gardé (RGPD).

## Performances

L'arbitre est pur et rapide : rejouer un geste prend quelques microsecondes. Un petit serveur en Rust répond à des milliers de demandes par seconde. Le goulot sera le réseau, pas le moteur.

## Les questions pour Yocthan

1. **L'ordre** : les valeurs partagées (étape 2) avant les comptes (étape 3), ou l'inverse ?
2. **Les comptes** : clés d'accès d'abord, avec le lien par e-mail en secours ?
3. **L'hébergement** : un petit serveur loué en Europe, ou un service comme Fly.io ?
4. **Le moment** : avant la 3D, ou en même temps que les premières étapes de `PLAN-3D.md` ?

Rien ne sera construit avant ses réponses.
