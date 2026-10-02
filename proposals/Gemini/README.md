# Proposition by Gemini — v0.1 : HoloFractal Core

## Contribution

* **Sujet :** Noyau spatial fractal, transition d'échelle continue, garantie d'empreinte mémoire bornée ($O(1)$ < 1 Go) et ontologie unifiée Nœud = Avatar = Monde.
* **Discussions sources :** `HC-001`, `HC-005`, `HC-007`
* **Décisions concernées :** `ADR-003`, `ADR-005`, `ADR-006`
* **Statut proposé :** `EXPÉRIMENTATION`
* **Implémentation :** Python 3.11 ou plus récent, bibliothèque standard stricte, sans dépendance externe.

---

## Hypothèse testée

> Si l'on unifie l'entité, le monde et le portail sous une primitive unique (le Nœud Fractal) et que l'on applique une règle de congélation en imposteurs au franchissement des seuils d'échelle, il est possible de naviguer dans une infinité de mondes imbriqués sans jamais dépasser un budget mémoire fixe (< 1 Go), à coût mémoire actif constant $O(1)$.

Ce prototype déplace la recherche : plutôt que de simuler une pièce 3D fermée avec des règles de domotique textuelles, il pose la fondation du **Web spatial fractal** directement adapté aux contraintes physiques des smartphones.

---

## Le modèle architectural

```text
               [ Big Bang : Nœud Racine N0 ]
                       /           \
           [ Nœud N1: Sphère ]     [ Nœud N1': Lettre A ]
                   |                       |
           (Seuil franchi)         (Seuil franchi)
          [ Sous-Monde N2 ]       [ Sous-Monde N2' ]
          (N0/N1 gelés en         (N0/N1 gelés en
             imposteurs)             imposteurs)

```

1. **L'Isomorphisme Ontologique :** Tout élément dérive de `FractalNode`. Il possède une enveloppe visuelle (aspect extérieur), un volume interne (monde potentiel) et un rayon de seuil métrique de transition.
2. **Le Sas Fractal Mémoire :**
   * Lorsque la caméra s'approche d'un nœud à une distance inférieure à son seuil de transition `scale_threshold`, le nœud intérieur devient le contexte actif.
   * Le nœud parent est instantanément congelé en **imposteur** (proxy basse fidélité de taille fixe, ex. 64 Ko).
   * L'empreinte mémoire active reste invariante par rapport à la profondeur de navigation.
3. **Contrôle Budgétaire Inviolable :** Le gestionnaire de mémoire (`MemoryLedger`) comptabilise les octets de géométrie, d'état et de textures. Si une création dépasse le quota configuré (ex. 1 Go au total), l'opération est formellement rejetée avec l'exception `MemoryQuotaExceededError`.
4. **Réactivité Déclarative Zéro-Code :** Les objets ne contiennent pas de code impératif libre. Ils s'associent par des liaisons Cause $\to$ Effet (Signaux $\to$ Actions), directement manipulables au doigt ou générables par IA.

---

## Comparaison avec les propositions existantes

| Critère | GPT-5.6 (PR #1) | Claude (PR #2) | Gemini (v0.1) |
| --- | --- | --- | --- |
| **Forme du Monde** | Boîte 3D fermée | Salles isolées (Espaces) | **Arborescence fractale continue** |
| **Gestion du 1 Go** | Aucune | Aucune | **Comptabilité stricte au niveau de l'octet + Sas d'imposteurs** |
| **Modèle d'accès** | Code textuel impératif | DSL avec vérificateur de types | **Nœuds déclaratifs réactifs (Zéro-Code ready)** |
| **Passage à l'échelle** | Dégradation $O(N)$ | Grille spatiale locale | **Stabilité mémoire $O(1)$ à travers la profondeur d'échelle** |

---

## Objections et limites

1. **Simulation de mémoire en octets théoriques :** En Python pur sans GPU réel, les coûts mémoire de la géométrie et des textures sont calculés selon la taille réelle des buffers compressés ASTC et sommets quantifiés, et non par allocation physique VRAM Metal/Vulkan. Le pilote GPU réel reste le juge de paix.
2. **Perte de parallaxe de l'imposteur :** Quand le monde parent est congelé en imposteur cubique, l'utilisateur situé à l'intérieur voit l'extérieur comme une skybox. Si la transition n'est pas masquée par un shader d'horizon ou un portail géométrique net, un artefact visuel de platitude apparaît.
3. **Non-Turing complétude volontaire :** Le modèle réactif sans boucles libres interdit certains algorithmes procéduraux complexes au runtime. C'est un compromis assumé pour garantir l'absence de freeze et de plantage sur smartphone.

---

## Expérience ou preuve requise

1. Exécuter la suite de tests unitaires `test_fractal.py` sur Python 3.11+.
2. Vérifier que lors d'une traversée de 5 niveaux de poupée russe spatiale ($N_0 \to N_1 \to \dots \to N_4$), la mémoire active ne présente aucune croissance linéaire, mais reste strictement stable en $O(1)$.
3. Valider le rejet systématique des entités dont le poids dépasse le budget mémoire étanche configuré.

## Lancer les tests

Depuis le dossier `proposals/Gemini/` :

```bash
python -m unittest test_fractal.py -v

```

---

## Documents à mettre à jour

Si cette orientation est retenue ou intégrée dans les décisions :

* `docs/01-holocode/PARADIGME-HOLOSCENIQUE.md` : Introduire l'isomorphisme du Nœud Fractal et le concept de seuil d'échelle (`scale_threshold`).
* `docs/02-gouvernance/DECISIONS.md` : Ouvrir un ADR sur le budget mémoire matériel garanti comme règle de typage (`ADR-007 : Quotas mémoires inviolables et sas d'imposteurs`).
* `docs/04-roadmap/ROADMAP.md` : Ajouter la validation du pipeline de zoom fractal mobile avant le réseau multi-joueur.
