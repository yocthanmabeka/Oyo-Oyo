# Passation du 2026-10-08 : si la session Claude du PC s'arrête

Ce document est pour Gemini (par Antigravity), ChatGPT, Codex, ou toute IA qui reprend le projet quand la session Claude du PC de Yocthan s'arrête, par exemple quand son quota est épuisé.

Décision de Yocthan, le 2026-10-08 : « reprendre à la main sans pour autant commettre de fautes, ni pour autant qu'ils fassent des fusions. […] C'est toi qui seras le seul à faire de fusion. » Le « toi » est la session Claude du PC de Yocthan.

## Les règles, sans exception

1. **Tu ne fusionnes jamais.** Cela vaut pour toute pull request, même la tienne, même avec tous ses tests verts. Seule une session Claude fusionne, après avoir relu : celle du PC de Yocthan tant qu'il tourne, celle du nuage (sur son téléphone) quand le PC est éteint ou en veille.
2. **Tu ne touches jamais à `main`.**
   - Pas d'envoi direct (`git push origin main`), pas d'envoi forcé (`--force`).
   - Pas de réécriture de l'histoire (`rebase` d'une branche déjà envoyée, `reset` d'une branche partagée).
   - Pas de branche effacée.

   `main` est protégée sur GitHub : un envoi direct y est refusé, et une pull request n'y entre qu'avec ses trois tests verts.
3. **Tu travailles dans ton propre dossier.** Jamais dans le dossier principal, `C:\Users\mokea\Documents\IA_creation\Metaverse` : le serveur que Yocthan regarde (port 8080) tourne depuis lui. Tu le crées ainsi, puis tu ouvres `_voir\GEMINI` :

   ```
   cd C:\Users\mokea\Documents\IA_creation\Metaverse
   git fetch origin
   git worktree add _voir\GEMINI -b gemini/<sujet> origin/main
   ```

   Pour reprendre un lot déjà commencé par Claude, pars de sa branche, mais sur une branche à toi : `git worktree add _voir\GEMINI -b gemini/lot6 origin/langage/lot6-partage`. Tu n'envoies jamais rien sur une branche de Claude.
4. **Une branche par travail, et une pull request en brouillon** : `gh pr create --draft`. Sa description dit :
   - ce qui est fait et ce qui reste ;
   - les commandes réellement exécutées, avec leurs résultats exacts ;
   - ce qui n'a pas été exécuté.

   Un résultat que tu n'as pas obtenu toi-même ne s'écrit jamais.
5. **Tu prends ton travail dans la file** : la plus ancienne issue `etat:a-prendre` qui est libre, selon la règle d'`AGENTS.md` (« La file de travail »). Ou bien ce que Yocthan te donne en te nommant.
   - Tu ne changes jamais le statut d'une décision (`ADR`). Une nouvelle décision est une `PROPOSITION`.
   - Tes numéros de décision et de leçon viennent de la réserve écrite dans le tableau « Qui fait quoi » d'`AGENTS.md`. Relis-le sur `origin/main` (`git show origin/main:AGENTS.md`).
6. **L'identité des enregistrements** : `git -c user.name="yocthanmabeka" -c user.email="190955222+yocthanmabeka@users.noreply.github.com" commit …`. Le dépôt est public : jamais d'adresse e-mail personnelle.
7. **Avant de demander une relecture** : dans `moteur/`, `.\outils\build.ps1` (les tests, la construction, `web/pkg` et `web/pkg-light`), puis `node outils/browser-tests.mjs` (les leçons et les gestes, dans Chrome sans fenêtre). Tout doit être vert.
   - N'installe rien sans l'accord de Yocthan.
   - N'ouvre aucune nouvelle fenêtre sur son PC : c'est sa batterie.
8. **Le code est en anglais, les commentaires et les textes en français** (`ADR-060`). Un ajout au langage arrive dans la même pull request que :
   - sa leçon (`exemples/lecons/`) ;
   - le guide (`docs/01-holocode/GUIDE.md`) ;
   - `NOMS.md` et `DECISIONS.md`.

   Avant d'ajouter un mot, compare les options, et vérifie qu'on ne répète pas un défaut de HTML, de CSS ou de JavaScript.
9. **N'écris ni dans `AGENTS.md`, ni dans le journal, ni dans `TABLEAU-WEB.md`** : Claude les tient. Ton compte rendu va dans la description de ta pull request.
10. **Ne déverrouille jamais le téléphone de Yocthan.** S'il est verrouillé, attends-le.

## Où en est le projet (le 2026-10-08, vers 2 h 30)

- **`main`** est au vert. Dernières fusions : PR 173 (lot 5, des adresses qui portent des valeurs), PR 175 (coordination), PR 176 (mesures du téléphone).
- **Les lots 1 à 5 et 8 du web** sont faits.
- **Lot 6, des valeurs partagées en direct**, gardées par le serveur pour tout le monde (décision de Yocthan) :
  - branche `langage/lot6-partage` ; numéros `ADR-079` et `ADR-080`, leçons 101 à 103 ;
  - fait et fusionné (PR 179) ; ses dettes sont dans `ADR-079`.
- **Lot 7, des comptes chez l'auteur**, par un mot de passe et un code à 6 chiffres, puis des clés d'accès :
  - branche `langage/lot7-comptes`, PR 177 ; numéros `ADR-081` à `ADR-083`, leçons 104 à 107 ;
  - construit, 166 tests Rust et 34 essais dans Chrome verts sur le PC ; à fusionner par la session Claude du PC, après avoir résolu ses conflits avec le lot 6 (`server.rs`, `page-engine.js`, `browser-tests.mjs`, le renvoi des touchers à unifier avec celui du lot 6).
- **Lot 9, et les tâches confiées le 2026-10-08** (les secondes et un chronomètre, des polices libres, l'historique dans une page, trois dettes) : à la session Claude du nuage. Ses PR 171, 172 et 174 sont ouvertes : n'y touche pas.
- **PR 153 (Codex)** : fusionnée le 2026-10-08. C'est une proposition, elle ne décide rien.
- **La file de travail** : les issues 180 à 189. Ce qui reste est marqué `etat:a-prendre`, et ce qui est déjà pris est marqué `etat:en-cours`.
- **Les mesures du téléphone** (vitesse, mémoire, batterie) sont remises : le câble a pris l'humidité. La marche à suivre est dans `proposals/Claude/telephone-2026-10-07/README.md`.
- **Après le lot 9** vient la 3D (`docs/04-roadmap/PLAN-3D.md`, à valider par Yocthan).
- **Ce qui attend une décision de Yocthan** :
  - remplir une page de profil par `Data(from: "profils/{id}.json")` ;
  - la validation des `ADR-062` à `ADR-069` et de l'`ADR-078` ;
  - le plan de la 3D ;
  - les bibliothèques `p256` et `sha2`, pour les clés d'accès (issue 181).

## Ce que tu peux faire sans risque

- Relire une pull request et la commenter. Un commentaire est un avis, jamais une décision.
- Continuer un travail que Yocthan t'a donné, dans ton dossier, sur ta branche, avec une pull request en brouillon.
- Lancer les essais et rapporter leurs résultats exacts.
- Expliquer à Yocthan, simplement, ce que fait une pull request.

## Ce que tu ne fais jamais

- Fusionner.
- Envoyer sur `main` ou sur une branche de Claude, forcer un envoi, effacer une branche, réécrire l'histoire.
- Changer le statut d'une décision.
- Installer un programme.
- Travailler dans le dossier principal ou arrêter son serveur 8080.
- Utiliser une adresse e-mail personnelle de Yocthan.
- Prendre un lot que Yocthan ne t'a pas donné.
