# HoloCode pour VS Code

Fait reconnaître les fichiers `.holo` par VS Code.

- **Les couleurs du langage** : blocs (`Page`, `P`, `Point`), paramètres (`name:`), styles (`.card { … }`), textes, nombres avec leur unité, couleurs, commentaires.
- **Les commentaires et les parenthèses** : `Ctrl+/` commente une ligne ; les parenthèses, crochets et accolades se referment seuls.
- **La faute, soulignée à sa place** (ADR-046) : le moteur vérifie le fichier pendant qu'on écrit, même pas encore enregistré, et souligne en rouge ce qu'il refuse, avec son message. Quand il dit le bon mot (« écris « H1 » »), ou qu'un mot connu est tout proche (`Butten` → `Button`), l'ampoule (ou `Ctrl+.`) propose **« Remplacer « h1 » par « H1 » »** : un clic, et c'est corrigé. Le moteur reste strict : rien n'est corrigé sans ce clic.
- **Les mots du langage proposés** : les blocs, les réglages, les valeurs du fichier ; après `Ajouter.`, les signaux ; après `panier.`, les demandes.
- **Un bouton ▶ en haut à droite** d'un fichier `.holo` (ou `Ctrl+Alt+H`, ou clic droit dans l'explorateur) : enregistre le fichier et l'ouvre dans le navigateur, à sa propre adresse. Le serveur local doit tourner : `node moteur/outils/server.mjs`.

## Installer

```
python outils/vscode-holocode/package.py
code --install-extension outils/vscode-holocode/holocode-0.2.1.vsix
```

Puis recharger la fenêtre de VS Code. Pour souligner les fautes, l'extension se sert du moteur construit dans le dépôt : `cargo build --release --bin holo` dans `moteur/`, une fois.

Le même éditeur existe dans le navigateur, sur le PC et sur le téléphone : `http://localhost:8080/editor?key=…` (l'adresse exacte est affichée par le serveur à son démarrage).

## Versions

- **0.2.1** (2026-10-08) : le bouton ▶ et `Ctrl+Alt+H` remarchent (ils étaient cassés depuis la traduction du moteur en anglais, `ADR-060`, et ont été réparés avec le lot 5) ; un modèle d'adresse (`profil/{id}.holo`, `ADR-078`) est vérifié comme les autres fichiers.
- **0.2.0** : la faute soulignée à sa place et la correction d'un clic (`ADR-046`).

## Limites

- Le bouton ▶ n'ouvre que les fichiers rangés dans `exemples/` ou dans `moteur/mondes/` : ce sont les deux dossiers que le serveur local sait servir.
- Le moteur ne donne que la première faute : une fois corrigée, la suivante apparaît.
