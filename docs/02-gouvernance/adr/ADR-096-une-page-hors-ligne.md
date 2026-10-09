# ADR-096 — Lot 9 : une page lisible hors-ligne, `Offline`

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09 : intégrer le travail de Codex quand il est bien fait)
- Date : 2026-10-08 (la construction, par Codex) ; 2026-10-09 (la relecture, deux corrections, l'intégration et la validation)
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md` : « le hors-ligne […] à la fin, avant la 3D ») ; la reprise du lot 9 par Codex (issue 202) ; la PR 203 et son compte rendu (`proposals/GPT5.6/fin-lot9-2026-10-08/README.md`) ; `holo serve` sans JavaScript (`ADR-074`) ; les valeurs partagées en direct (`ADR-079`) ; les comptes (`ADR-081`).
- Validation : Yocthan, le 2026-10-09.
- Projets affectés : HoloCode, HoloEngine, le serveur
- Auteur de la construction : **Codex** (PR 203) ; relu, corrigé, intégré et documenté par Claude. Voir aussi `ADR-093`, `ADR-094`, `ADR-095`.

## Contexte

Une page qu'on relit dans le métro, sans réseau. Sur le web, il faut écrire un service worker et gérer `CacheStorage` à la main : c'est l'un des codes les plus fragiles du web (une page privée gardée dans le cache, une vieille version servie pour toujours, un formulaire « mis en attente » qui part deux fois).

## Décision

```holo
Page(
  state: State(count: 0),
  children: [
    H1("My public notebook"), P("Local counter: {count}"),
    Button(name: Add, text: "Add one"),
    Offline(name: Copy, label: "Public copy in this browser", files: ["image.svg"]),
    Button(name: Save, text: "Keep an offline copy"),
    Button(name: Remove, text: "Delete the offline copy"),
  ],
  rules: [ On(Add.tap, effect: count.add(1)), On(Save.tap, effect: Copy.save), On(Remove.tap, effect: Copy.remove) ],
)
```

1. **`Offline(name:, label:, files:)`** : `Copy.save` garde une copie de la page dans le navigateur du visiteur ; `Copy.remove` l'efface. Seulement sur le toucher d'un bouton.
2. **Ce que la copie prend** : la page à son adresse sans paramètres, son texte `.holo`, la porte d'entrée et le moteur léger, et les fichiers nommés dans `files` (16 au plus, du site : des images par exemple, jamais d'autres pages, scripts ou modules). 16 Mio par copie, huit pages au plus.
3. **Le réseau d'abord** : la copie ne sert que si le réseau manque. Une erreur du serveur reste une erreur.
4. **Seule une page publique se copie** : pas d'accès réservé, de compte, de valeurs partagées, de valeurs gardées (`keep`), de données reçues, d'import, de formulaire, de module, de monde, de son, de vidéo, ni d'autre capacité. Une réponse privée (`private`, `no-store`) ou redirigée est refusée ; une adresse de compte (`/account`) n'est jamais copiée.
5. **La copie est celle d'un premier visiteur** : le service worker la demande sans cookie, et le serveur, qui ne sait pas qui la demande, rend la page de départ. Rechargée hors-ligne, elle repart de ses valeurs de départ ; les changements de la visite restent locaux.
6. **Rien n'est mis en attente** : aucune écriture n'est retenue pour être envoyée plus tard, aucune n'est rejouée.
7. **Sans JavaScript, une page comme les autres** (correction de la relecture) : avec `holo serve`, une page qui déclare `Offline` est servie comme toute page, avec les valeurs du visiteur, et ses boutons partent par le formulaire des gestes (`ADR-074`). La première version la servait toujours avec ses valeurs de départ, en dehors du chemin des pages : sans JavaScript, un toucher n'y changeait rien de visible.
8. **Le service worker ne touche ni aux écritures ni au direct** (correction de la relecture) : un geste, un formulaire, une connexion (tout ce qui n'est pas `GET`) et le direct des valeurs partagées (`Accept: text/event-stream`, `ADR-079`) vont au réseau sans passer par lui. La première version faisait passer le direct par le service worker, et l'aurait servi depuis une copie (une page HTML à la place du flux) si le réseau manquait.
9. Le service worker (`holo-capabilities-worker.js`) ne s'installe que lorsqu'un visiteur demande une copie ou une notification (`ADR-095`) ; il vaut alors pour tout le site.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Quoi copier | tout le site ; **une page, à la demande du visiteur** | le visiteur choisit ce qu'il garde ; la place est bornée et se voit |
| Le réseau ou la copie d'abord | la copie d'abord (rapide, mais vieille) ; **le réseau d'abord** | une page à jour quand c'est possible ; la copie seulement en secours |
| Une page privée | la copier ; **la refuser** | une copie privée resterait sur un appareil partagé |
| Les envois hors-ligne | les retenir et les rejouer ; **les refuser** | un envoi rejoué deux fois, ou des semaines plus tard, est pire qu'un envoi refusé tout de suite |
| La copie du serveur | une adresse à part ; **la page demandée sans cookie** | aucun chemin spécial dans le serveur : la page de départ est celle que reçoit tout premier visiteur |

## Les défauts du web évités

- **La page privée gardée dans le cache** : ici, refusée (une réponse privée, un compte, des valeurs partagées), et la copie est demandée sans cookie.
- **La vieille version servie pour toujours** : ici, le réseau passe d'abord.
- **Le formulaire mis en attente qui part deux fois** : ici, rien n'est retenu.
- **Le service worker qui avale le direct** (un `EventSource` servi par le cache, coupé quand le worker s'arrête) : ici, il le laisse passer.

## Dettes

- Une page qui a un compte, un formulaire ou des valeurs partagées ne se copie pas ; un envoi hors-ligne n'existe pas.
- Une page servie sans `.holo` dans l'adresse (`/contact`) ne se copie pas encore.
- Le navigateur peut effacer la copie s'il manque de place ; un remplacement qui échoue retire la copie incomplète, sans garder l'ancienne.
- La vérification « ne lit pas un compte » cherche les mots `account` et `signedIn` dans tout le fichier : un texte qui contient « account » est refusé à tort.
- À voir sur le téléphone : la place prise, la batterie, le rechargement réel sans réseau.

## Critères de validation

- Test du serveur : `an_offline_page_keeps_its_values_without_javascript` (deux touchers sans JavaScript, la page montre 2 ; la page demandée sans cookie montre 0, sans en-tête privé ni cookie).
- Dans Chrome : « lot9 : copie hors-ligne, rechargement réel sans réseau et effacement » (de Codex) ; « holo serve : une page hors-ligne sans JavaScript ; le service worker laisse le direct et les écritures » (sans JavaScript, deux touchers gardés ; avec, les mêmes valeurs ; la copie prête, avec les valeurs de départ ; puis, sous le service worker, la page servie par lui, le direct et un geste partagé passés à côté). L'essai rate avec l'ancien service worker.
- Leçon `119-une-page-hors-ligne.holo`.
