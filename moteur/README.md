# Moteur HoloCode — sprint Big Bang

- Statut : `EXPÉRIMENTATION`
- Décisions mises à l'épreuve : `ADR-005` (téléphone, navigateur, 1 Go), `ADR-007` (vue en profondeur), `ADR-008` (même fichier, même résultat), `ADR-009` (format `.holo`), `ADR-010` (moteur Rust, WebAssembly, `wgpu`)
- Auteur : Claude, à la demande de Yocthan, le 2026-10-03

## Ce que c'est

La première preuve exécutable de la vision de Yocthan : un point lumineux qui se morcelle en points quand on zoome, et dans chaque point un monde, qui contient lui-même des points, sans fin. Tout est piloté par un fichier de huit lignes, [`mondes/big-bang.holo`](mondes/big-bang.holo) :

```holo
Point(
  nom: Origine,
  graine: 1,
  lumiere: 1.0,
  morceler: 12,
)
```

Rien n'est stocké : chaque monde se calcule à partir de sa graine, et chaque point enfant reçoit une graine dérivée de celle de son parent. Descendre de mille niveaux coûte mille graines de 8 octets. Le même fichier donne le même univers sur toutes les machines.

Le moteur est écrit en Rust, compilé en WebAssembly, et dessine avec `wgpu` : WebGPU quand le navigateur l'offre, WebGL 2 sinon. Le même code pourra plus tard tourner en natif, dans le navigateur propre au projet.

## Comment on s'en sert

- **Pincer** (ou la molette) : zoomer. Le point se morcelle, puis on s'approche du point visé, on aperçoit le monde qu'il contient, et on y entre.
- **Toucher** un point : il devient le point visé, signalé par un halo blanc qui respire ; c'est dans lui que l'on entre en zoomant. Tant qu'on n'a touché aucun point, la cible est celui qui est au centre.
- **Glisser** avec un doigt : tourner le monde.
- **Pincer dans l'autre sens** : dézoomer jusqu'à ressortir dans le monde parent.
- **Pause** : le bouton « pause », en bas à droite, arrête tout calcul et tout dessin ; « reprendre » relance. Le monde se met aussi en pause tout seul quand l'onglet est caché.
- Les mesures sont cachées : le petit bouton « mesures », en bas à droite, les affiche avec le bouton « Copier le rapport » (ou `?mesures=1` dans l'adresse).

## Construire et lancer

Il faut Rust (avec la cible `wasm32-unknown-unknown`) et Node.

```powershell
cd moteur
.\outils\construire.ps1        # cargo test, cargo build (wasm), wasm-bindgen → web/pkg
node outils/serveur.mjs        # http://localhost:8080
```

Le script télécharge `wasm-bindgen` 0.2.100 dans `outils/bin/` s'il manque. Le serveur compresse en Brotli, comme un vrai hébergement, pour que le poids transféré affiché soit le vrai.

Paramètres d'adresse utiles : `?zoom=3.4` démarre à un zoom donné (pour les captures), `?monde=nom` charge `mondes/nom.holo`.

## Tester sur le téléphone

1. Lance le serveur sur le PC ; il affiche une adresse `http://192.168.x.x:8080` (ou `10.x.x.x`).
2. Sur le téléphone, connecté au même Wi-Fi, ouvre cette adresse dans Chrome.
3. **WebGPU exige une page « sécurisée »** : en `http://` sur une adresse locale, Chrome ne l'offre pas, et le moteur tourne en WebGL 2. C'est déjà une mesure utile. Pour mesurer WebGPU, au choix :
   - sur le téléphone, ouvre `chrome://flags/#unsafely-treat-insecure-origin-as-secure`, ajoute l'adresse du PC (par exemple `http://192.168.1.20:8080`), active, relance Chrome ;
   - ou branche le téléphone en USB avec le débogage activé, et dans Chrome sur le PC, `chrome://inspect` → « Port forwarding » : `8080` → `localhost:8080`. Sur le téléphone, `http://localhost:8080` est alors sécurisé.
4. Laisse tourner 15 minutes en zoomant et dézoomant, puis appuie sur « Copier le rapport » et colle le résultat dans une discussion ou dans le dépôt.

Le rapport contient : le moteur de rendu utilisé, les images par seconde, la pire image en millisecondes, le temps jusqu'à la première image, le poids du moteur transféré et réel, la mémoire JavaScript (Chrome), la profondeur atteinte, et la batterie au départ et à l'arrivée.

## Ce qui est mesuré sur le PC

| Mesure | Résultat | Cible du sprint |
|---|---|---|
| Tests du cœur (`cargo test`, Rust natif) | 17 sur 17 | |
| Poids du moteur WebAssembly, réel | 1 893 Ko | |
| Poids transféré (Brotli) | **489 Ko** (+ 14 Ko de JavaScript) | moins de 2 Mo |
| Fichier de la page HTML | 1 Ko, le même pour tous les mondes | |
| Fichier `.holo` du monde | 8 lignes | |
| Repli WebGPU → WebGL 2 | Vérifié : sans carte graphique, le moteur bascule tout seul | |
| Parcours complet en captures d'écran | point entier → morcellement → plongée avec aperçu du monde intérieur → entrée (« Origine › 7 ») | |

Les images par seconde et la mémoire mesurées sur le PC sans carte graphique (rendu logiciel) n'ont aucun sens et ne sont pas reportées. **Les mesures qui comptent sont celles du téléphone, et elles ne sont pas encore faites.**

## Captures (Chrome sans fenêtre, rendu WebGL 2 logiciel)

| Zoom 0 | Zoom 0,8 | Zoom 3,4 | Zoom 4,6 |
|---|---|---|---|
| ![le point entier](captures/1-point-entier.png) | ![morcellement](captures/2-morcellement.png) | ![plongée, monde intérieur visible](captures/3-plongee-apercu-du-monde-interieur.png) | ![entré, profondeur 2](captures/4-entre-profondeur-2.png) |

## Critères d'abandon du sprint

Repris de la proposition de Gemini, à vérifier sur le téléphone :

| Mesure | Cible | Abandon si |
|---|---|---|
| Fluidité | 60 images par seconde | saccades de plus de 50 ms à l'entrée dans un point |
| Mémoire de l'onglet | moins de 300 Mo, stable avec la profondeur | elle croît avec la profondeur |
| Première image | moins de 1 seconde | |
| Batterie | | plus de 20 % perdus en 15 minutes |

## Limites et choix à discuter

- **Le lecteur `.holo` ne lit que `Point`.** Il sait déjà lire toute la grammaire brouillon (blocs, listes, textes Markdown, unités, imports) et refuse le code libre dans un bloc, mais seul `Point` reçoit un sens. `Page`, `Texte`, `Bouton` et la vue à plat viendront ensuite.
- **Les positions des points dépendent de sinus et cosinus.** Les entiers (graines, nombres de points, couleurs) sont identiques partout ; les positions pourraient différer d'un milliardième entre un PC et un téléphone. Ce sera invisible, mais ce n'est pas strictement « même fichier, même résultat » : à régler si l'on veut des mondes partagés au bit près.
- **La transition d'entrée est un fondu**, pas une continuité parfaite : le monde intérieur aperçu avant d'entrer et le monde affiché après ne coïncident pas exactement.
- **Aucune loi ni phénomène** : ce sprint ne porte que sur la navigation et le poids.
- **Densité de pixels plafonnée à 2** pour ménager la chauffe du téléphone.
- `wgpu` est fixé à la version 24 ; une montée de version demandera quelques retouches.

## Fichiers

| Fichier | Rôle |
|---|---|
| `src/holo.rs` | Lecture d'un fichier `.holo` : mots, blocs, erreurs avec ligne et colonne |
| `src/graine.rs` | Graines et nombres pseudo-aléatoires reproductibles, en arithmétique entière |
| `src/univers.rs` | Du bloc `Point` au monde ; les enfants et leurs graines |
| `src/navigation.rs` | Morcellement, zoom, entrée, sortie ; produit la liste des disques à dessiner |
| `src/rendu.rs`, `src/rendu.wgsl` | Dessin avec `wgpu` : un disque lumineux par instance, mélange additif |
| `src/web.rs` | Zone de dessin, doigts, molette, boucle d'affichage, mesures dans `window.__holo` |
| `web/index.html` | La seule page HTML, générée une fois pour tous les mondes |
| `web/mesures.js` | L'affichage des mesures et le bouton « Copier le rapport » |
| `outils/serveur.mjs` | Serveur local avec compression Brotli |
| `outils/construire.ps1` | Construction complète |
