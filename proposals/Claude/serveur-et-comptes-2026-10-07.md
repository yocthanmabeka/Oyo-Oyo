# Proposition : un serveur en Rust qui fait tourner le même arbitre, puis des comptes

- Auteur : Claude, d'après une question de Yocthan
- Date : 2026-10-07
- **Statut : proposition, à discuter avec Yocthan.** Rien n'est construit. Yocthan, le 2026-10-07 : « on veut un langage qui construit le métaverse de A à Z […] fluide, limpide ».
- Révisée le même jour, après la règle de Yocthan : chez soi d'abord, aucun prestataire obligatoire.

## La règle de Yocthan : chez soi d'abord

> « je n'aimerais pas lancer un langage avec des autorisations sur Google, sur Apple ou bien sur Microsoft, sans pour autant que l'utilisateur lui-même soit libre […] Je ne suis pas obligé d'aller chez Google ou bien chez un prestataire quelconque pour une version d'un site qui tourne d'abord dans mon ordinateur. Mais par après, c'est là que j'y vais […] mais pas dès le départ. » (Yocthan, 2026-10-07)

Cette proposition suit cette règle :

- Tout tourne d'abord sur le PC de l'auteur, sans Internet et sans compte nulle part.
- Les données sont dans un fichier du dossier du site.
- Les comptes sont gérés par le serveur de l'auteur, dans sa propre base.
- Un prestataire (un hébergeur, un service d'e-mails, « Se connecter avec Google ») n'est jamais obligatoire : l'auteur le choisit plus tard, s'il le veut.

C'est déjà vrai pour ce qui existe aujourd'hui :

- `node outils/server.mjs` sert les pages sur le PC (et sur un téléphone du même Wi-Fi) ; les messages des formulaires vont dans `messages/`, les fichiers envoyés dans `messages/files/`. Rien ne part ailleurs.
- Le moteur refuse de charger une police, une image, un son, une vidéo ou un module depuis un autre site : chaque fichier est rangé à côté de la page (`Font(source:)`, `Image(source:)`, `Sound(source:)`…). Une page HoloCode n'appelle donc ni Google Fonts, ni un réseau de diffusion, ni un traceur. Seuls les liens (`A(to:)`) peuvent mener ailleurs, quand le visiteur les touche.

### Comme Django

| | Django (Python) | HoloCode (proposé) |
|---|---|---|
| Lancer le site sur son PC | `python manage.py runserver` | `holo serve` |
| La base de données | un fichier `db.sqlite3` dans le projet | un fichier SQLite dans le dossier du site |
| Les comptes | intégrés (`django.contrib.auth`), mots de passe chiffrés | intégrés au moteur, dans ta base |
| La double authentification | un paquet libre à ajouter (`django-otp`), sans prestataire | intégrée : un code à 6 chiffres, sans prestataire |
| Les e-mails pendant le développement | affichés dans le terminal | le lien de connexion affiché dans le terminal |
| Google, Apple, GitHub | en option, plus tard | en option, plus tard, si l'auteur le veut |

## La question de Yocthan sur Node.js

> Les langages issus de JavaScript n'utilisent-ils pas vraiment Node.js pour gérer le serveur ou les comptes ?

**Oui.** Dans le monde JavaScript, la page tourne dans le navigateur, et le serveur tourne presque toujours dans Node.js (avec Express, Next.js, SvelteKit, Nuxt), ou chez un service tout fait (Firebase, Supabase). L'auteur écrit donc deux programmes : celui de la page et celui du serveur. Avec TypeScript des deux côtés, c'est le même langage, mais ce sont deux programmes qu'il faut garder d'accord. C'est là que naissent beaucoup de failles : la page vérifie une chose, le serveur en oublie une autre.

HoloCode a aujourd'hui un petit serveur d'essai en Node.js (`moteur/outils/server.mjs`) : il sert les pages, range les messages et les fichiers. Il est fait pour le PC de Yocthan, pas pour Internet.

## L'idée : un seul moteur, des deux côtés

Le cœur de HoloCode est déjà un **arbitre pur** : un état et un geste donnent un nouvel état. Il est écrit en Rust et tourne dans le navigateur (compilé en WebAssembly). Le même code peut tourner sur un serveur, sans rien réécrire.

```
          navigateur                                serveur (Rust)
   page.holo ─► arbitre ─► HTML          page.holo ─► arbitre ─► ce qui est gardé
                  │                                     ▲
                  └──── « Like.tap » (un geste) ────────┘
```

- **L'auteur n'écrit pas de serveur.** Il écrit sa page ; il dit, en HoloCode, ce qui est gardé et partagé, et qui peut le voir.
- **La page envoie des gestes, pas des valeurs.** Le serveur rejoue le geste avec le même arbitre, sur sa propre copie : il ne croit jamais ce que dit le navigateur. C'est déjà ce que fait l'envoi d'un fichier (`ADR-059`) : le serveur demande au moteur ce que la page permet.
- **Un seul programme, chez toi d'abord** : `holo serve dossier/` sert le site, fabrique les pages, reçoit les formulaires et les fichiers, garde les données. Sur le PC, il remplace `node outils/server.mjs`.

Pour la fluidité que veut Yocthan (« comme si on faisait lire deux films », ce qu'on ne veut pas) : il n'y a qu'un langage, qu'un moteur, qu'une vérité. La page et le serveur ne peuvent pas diverger, puisque c'est le même arbitre.

## Qu'est-ce qui est « partagé » ?

Aujourd'hui, chaque valeur d'une page vit dans le navigateur du visiteur. Si Ada touche « J'aime », son compteur passe à 1 ; Bob, sur son téléphone, voit toujours 0. Une valeur **partagée** est gardée par le serveur : elle est la même pour tous ceux qui ouvrent la page.

| Écriture | Où vit la valeur | Qui la voit | Exemple |
|---|---|---|---|
| `State(likes: 0)` | dans l'onglet du visiteur ; elle repart à 0 au rechargement | lui seul | un compteur pour jouer |
| `keep: [likes]` (existe déjà) | dans le navigateur du visiteur, d'une visite à l'autre | lui seul, sur cet appareil | ses réglages, son meilleur score |
| `shared: [likes]` (proposé) | sur le serveur | tout le monde, la même valeur | des « j'aime », un livre d'or, le stock d'une boutique |
| `mine: [cart]` (proposé, avec les comptes) | sur le serveur, une par personne | chacun la sienne, sur tous ses appareils | son panier, sa progression |

L'exemple de la leçon 75 : Ada touche « Réserver » sous Sunrise. Aujourd'hui, seule Ada voit « Réservé » ; Bob voit encore le tableau disponible et peut le réserver aussi. Avec `shared: [reserve]`, Bob voit tout de suite « Déjà réservé ». Pour le métaverse, c'est la même chose : pour se voir l'un l'autre, la place de chaque personne doit être partagée.

## Par étapes

| Étape | Ce qu'on gagne | Écriture proposée (à discuter) | Séances |
|---|---|---|---|
| **1. `holo serve`** | Le serveur d'essai devient un vrai serveur en Rust, sur ton PC d'abord : pages, formulaires, fichiers | rien de nouveau pour l'auteur | 2 |
| **2. Des valeurs partagées** | Un tableau réservé une seule fois, un livre d'or, des « j'aime » vus par tous | `State(likes: 0, shared: [likes])` ; le serveur garde `likes` et arbitre les gestes de tous | 2 à 3 |
| **3. Des comptes, gérés par ton serveur** | Chacun a ses valeurs : son panier, ses tableaux, sa progression | `Page(account: optional)` ; `{account.name}` ; `If(account, …)` ; `State(cart: 0, mine: [cart])` | 3 à 4 |
| **4. En direct, à plusieurs** | Voir les autres bouger : le pont vers le métaverse à plusieurs (`PLAN-3D.md`) | les valeurs partagées arrivent d'elles-mêmes, sans recharger | 3 |

Chaque étape aurait sa leçon, son ADR, ses tests, comme d'habitude.

## Les comptes, gérés par ton propre serveur

Toutes ces façons de se connecter seraient dans le moteur, et toutes marchent sur le PC de l'auteur, sans Internet :

| Façon de se connecter | Ce que garde ton serveur, dans ta base | Un prestataire ? |
|---|---|---|
| **Mot de passe + code à 6 chiffres** | le mot de passe chiffré (Argon2) et le secret du code | aucun : le code, qui change toutes les 30 secondes, est calculé par une appli du visiteur (Aegis, FreeOTP, ou une autre), sans Internet ; c'est une norme ouverte (RFC 6238) |
| **Clé d'accès** | une clé publique, rien de secret | aucun : c'est une norme ouverte du web (WebAuthn, du W3C, comme le HTML) ; la clé secrète reste chez le visiteur |
| **Lien par e-mail** | l'adresse | sur le PC : le lien s'affiche dans le terminal, comme Django ; en ligne : ton propre serveur de mail, ou un service, au choix |
| **« Se connecter avec Google, Apple, GitHub »** | un identifiant | oui, le leur : plus tard, en option, seulement si l'auteur le veut |

- **Une clé d'accès n'est pas « Se connecter avec Google ».** Le site ne parle jamais à Google, à Apple ou à Microsoft. La clé secrète reste chez le visiteur : dans son téléphone, sur une clé USB de sécurité, ou dans un gestionnaire libre comme Bitwarden. Google ou Apple n'apparaissent que si le visiteur choisit lui-même de ranger sa clé dans leur trousseau.
- **Sur un téléphone, en Wi-Fi local**, les clés d'accès demandent une page sécurisée (https), comme WebGPU : on fera comme pour WebGPU (le câble USB, voir `moteur/README.md`, ou un certificat fait chez soi). Le mot de passe et son code marchent partout.
- Tout serait écrit en Rust, avec des bibliothèques libres.

Je propose **le mot de passe avec son code à 6 chiffres d'abord** (ce que tout le monde connaît, comme Django), **les clés d'accès juste après** ; l'auteur choisit dans sa page.

## En ligne, plus tard

Seulement le jour où l'auteur veut que d'autres personnes, ailleurs, ouvrent son site :

- **Les données** : le même fichier SQLite, sauvegardé chaque nuit. Assez pour des milliers de visiteurs.
- **La machine** : sa propre machine (un vieux PC, un petit ordinateur comme un Raspberry Pi) derrière sa box, ou un serveur loué (environ 5 € par mois, en Europe pour le RGPD). À choisir ce jour-là.
- **HTTPS** : un relais comme Caddy, qui obtient tout seul un certificat gratuit (Let's Encrypt).

## Sécurité, ce que le moteur garantirait

- Aucun code libre, côté page comme côté serveur : seulement ce que le moteur sait faire, borné.
- Le serveur rejoue chaque geste avec l'arbitre ; une valeur envoyée par le navigateur n'est jamais crue.
- Des limites partout : taille des envois, nombre de gestes par minute et par visiteur, place gardée par page et par compte.
- Les envois ne sont acceptés que depuis le site lui-même (même origine).
- Un compte peut être effacé, avec tout ce qu'il a gardé (RGPD).

## Performances

L'arbitre est pur et rapide : rejouer un geste prend quelques microsecondes. Un petit serveur en Rust répond à des milliers de demandes par seconde. Le goulot sera le réseau, pas le moteur.

## Libre comme l'open source ?

Aujourd'hui, le dépôt est privé et `moteur/Cargo.toml` dit `license = "UNLICENSED"` : légalement, HoloCode n'est pas encore libre. Le jour de la publication, il faudra choisir une licence, par exemple :

- **MIT ou Apache 2.0**, comme Rust : chacun fait ce qu'il veut du code, même le garder fermé après l'avoir modifié ;
- **GPL ou AGPL** : celui qui modifie HoloCode et le redistribue (et, avec l'AGPL, celui qui fait tourner un serveur modifié) doit partager ses changements.

C'est une décision de Yocthan ; rien ne presse.

## Les questions pour Yocthan

1. **L'ordre** : les valeurs partagées (étape 2) avant les comptes (étape 3) ? Claude le conseille : c'est plus simple (aucune identité à protéger), utile tout de suite (le tableau réservé une seule fois), et les comptes s'appuieront dessus.
2. **Se connecter** : le mot de passe avec son code à 6 chiffres d'abord, les clés d'accès juste après, tout chez toi ?
3. **Le moment** : les étapes 1 et 2 avant la 3D, et les comptes en même temps que la 3D ?

L'hébergement et la licence attendront le jour où tu voudras publier. Rien ne sera construit avant tes réponses.
