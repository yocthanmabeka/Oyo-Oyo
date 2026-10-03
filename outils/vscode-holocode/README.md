# HoloCode pour VS Code

Fait reconnaître les fichiers `.holo` par VS Code.

- **Les couleurs du langage** : blocs (`Page`, `P`, `Point`), paramètres (`name:`), styles (`.card { … }`), textes, nombres avec leur unité, couleurs, commentaires.
- **Les commentaires et les parenthèses** : `Ctrl+/` commente une ligne ; les parenthèses, crochets et accolades se referment seuls.
- **Un bouton ▶ en haut à droite** d'un fichier `.holo` (ou `Ctrl+Alt+H`, ou clic droit dans l'explorateur) : enregistre le fichier et l'ouvre dans le navigateur, à sa propre adresse. Le serveur local doit tourner : `node moteur/outils/serveur.mjs`.

## Installer

```
python outils/vscode-holocode/empaqueter.py
code --install-extension outils/vscode-holocode/holocode-0.1.1.vsix
```

Puis recharger la fenêtre de VS Code.

## Limites

- Le bouton ▶ n'ouvre que les fichiers rangés dans `exemples/` ou dans `moteur/mondes/` : ce sont les deux dossiers que le serveur local sait servir.
- Les erreurs du fichier ne sont pas soulignées dans l'éditeur. Elles s'affichent dans le navigateur, avec leur ligne. Les souligner demandera de brancher le vérificateur du moteur sur VS Code.
