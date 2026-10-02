# Proposition by Gemini — v0.1 : HoloFractal Core

## Contribution

- **Sujet :** Noyau spatial fractal, transition d'échelle continue, mémoire comptée sous un plafond déclaré (< 1 Go), constante au départ puis croissante de 64 Ko par niveau descendu et ontologie unifiée Nœud = Avatar = Monde.
- **Discussions sources :** `HC-001`, `HC-005`, `HC-007`, `HC-013`
- **Décisions concernées :** `ADR-003`, `ADR-005`, `ADR-006`, `ADR-007`
- **Statut proposé :** `EXPÉRIMENTATION`
- **Implémentation :** Python 3.11 ou plus récent, bibliothèque standard stricte, sans dépendance externe.
- **Corrections de Claude, le 2026-10-02, à la demande de Yocthan :** une ligne de `runtime.py` (entrée dans un nœud sans enfants), et dans ce README les affirmations que la mesure contredisait : mémoire « constante », quota par nœud, cohérence du registre, ligne « passage à l'échelle » du tableau. La section « Résultats mesurés » est ajoutée. Le reste est de Gemini.

---

## Avertissement méthodologique

Ce prototype vérifie une **logique de comptage et de gestion de quotas déclarés en mémoire abstraite**, et non une mesure physique d'occupation VRAM/RAM sur silicium mobile réel. Les coûts mémoire sont calculés à partir de représentations quantifiées (sommets compressés 16 octets, textures ASTC, structures d'état) et d'imposteurs de taille fixe (64 Ko). La validation physique finale relève d'un démonstrateur WebGPU exécuté sur smartphone.

---

## Hypothèse testée

> Si l'on unifie l'entité, le monde et le portail sous une primitive unique (le Nœud Fractal) et que l'on applique une règle de chargement à la demande avec congélation en imposteurs au franchissement des seuils d'échelle, il est possible de naviguer dans une arborescence de mondes imbriqués sans dépasser un budget mémoire déclaré (< 1 Go), avec un coût mémoire de départ indépendant de la taille de l'univers.

Ce prototype étudie la sémantique de navigation spatiale continue du **Web spatial fractal** : naviguer consiste à plonger à l'intérieur d'un nœud géométrique ou à en ressortir.

---

## Le modèle architectural

```text
               [ Big Bang : Nœud Racine N0 ]
                       /           \
           [ Nœud N1: Sphère ]     [ Nœud N1': Lettre A ]
                   |                       |
           (Seuil franchi)         (Seuil franchi)
          [ Sous-Monde N2 ]       [ Sous-Monde N2' ]
          (N0/N1' gelés en        (N0/N1 gelés en
             imposteurs)             imposteurs)
```

1. **L'Isomorphisme Ontologique :** Tout élément dérive de `FractalNode`. Il possède une enveloppe visuelle extérieure, un espace interne potentiel et un seuil métrique de transition (`scale_threshold`).
2. **Le Paging Fractal à la demande :**
   * À l'initialisation, seuls le contexte actif et ses enfants directs visibles sont enregistrés dans le registre mémoire. Les mondes plus profonds restent dormants (coût nul).
   * Lors de la pénétration dans un nœud (`zoom_into`), le parent et l'ensemble des nœuds frères sont congelés en imposteurs de taille fixe (64 Ko). Les enfants du nœud pénétré sont alors chargés dans le registre actif.
   * Lors de la sortie d'un sous-monde (`zoom_out`), les enfants du sous-monde sont déchargés, le parent et les frères sont dégelés à pleine résolution, et le signal `SCALE_EXIT` est émis.
3. **Contrôle Budgétaire Inviolable :**
   * **Plafond global de l'arène :** Le gestionnaire (`MemoryLedger`) rejette toute allocation menant à un dépassement du budget global (ex. 1 Go).
   * **Quotas individuels par nœud :** Chaque nœud déclare un plafond de triangles (`max_triangles`) et de mémoire (`max_memory_bytes`), vérifié à l'enregistrement, et seulement à ce moment-là : un nœud peut ensuite dépasser son quota en accumulant de l'état.
   * **Cohérence des mutations d'état :** Toute modification d'état faite par `set_node_state` réajuste aussitôt le solde du registre, qui ne devient plus négatif. Une écriture directe dans `node.state` n'est comptée qu'au prochain gel : le registre peut alors sous-estimer.
4. **Réactivité Déclarative Sans Code :** Les interactions s'associent par liaisons Cause $\to$ Effet (Signaux $\to$ Actions) déterministes sans boucles libres, incluant la mutation d'état, le morphing d'apparence et la propagation d'impulsions (`PULSE_SIGNAL`).

---

## Comparaison avec les propositions existantes

| Critère | GPT-5.6 (PR #1) | Claude (PR #2) | Gemini (v0.1) |
|---|---|---|---|
| Forme du Monde | Boîte 3D fermée | Salles isolées (Espaces) | Arborescence fractale continue |
| Gestion du 1 Go | Aucune | Aucune | Comptabilité stricte au niveau de l'octet + Sas d'imposteurs |
| Modèle d'accès | DSL textuel déclaratif (relations & phénomènes) | DSL déclaratif avec vérificateur de types & lois | Nœuds déclaratifs réactifs (Zéro-Code ready) |
| Passage à l'échelle | Non mesuré | Grille spatiale locale | Chargement à la demande : mémoire de départ constante, puis 64 Ko de plus par niveau descendu |

---

## Objections et limites

1. **Calcul théorique d'octets déclarés :** En Python pur, les coûts sont calculés sur des modèles théoriques et non sur des allocations de mémoire physique du système d'exploitation mobile.
2. **Perte de parallaxe de l'imposteur :** Quand le monde parent et les frères sont congelés en imposteurs cubiques, ils sont perçus sous forme de skybox/proxy simplifié, nécessitant un traitement graphique adapté pour masquer la transition.
3. **Non-Turing complétude volontaire :** L'interdiction des boucles libres et du code arbitraire empêche les algorithmes procéduraux complexes en cours d'exécution, au bénéfice de la stabilité du runtime sur mobile.

4. **La mémoire comptée n'est pas constante avec la profondeur :** l'imposteur de chaque ancêtre reste compté, soit 64 Ko de plus par niveau descendu. Pour une mémoire réellement constante, seuls le parent direct et ses frères ont besoin d'un imposteur ; les ancêtres plus lointains pourraient être déchargés.

---

## Résultats mesurés

Mesures faites par Claude en exécutant ce code sous Python 3.14. Ce sont des octets déclarés et additionnés par le registre, pas de la mémoire de téléphone.

| Mesure | Résultat |
|---|---|
| Tests unitaires | 8 sur 8, exécutés aussi automatiquement sur GitHub |
| Mémoire comptée au départ | 1 212 Ko, que l'univers ait 5, 50, 150 ou 2 000 niveaux |
| Pendant la descente | 64 Ko de plus par niveau : 1 980 Ko au niveau 12 |
| Descente puis remontée de 12 niveaux | Le registre revient exactement à sa valeur de départ |
| Univers de 2 000 niveaux | Descente complète sans plantage ; 124,9 Mo comptés au niveau 1 998 |
| Plafond de 1 Go | Atteint vers 15 000 niveaux descendus |
| Nœud au quota de 10 Mo | A accepté 100 000 variables d'état et pesait alors 12,2 Mo |

---

## Expérience requise

Exécuter la suite de tests unitaires sur Python 3.11+ :

```bash
python -m unittest test_fractal.py -v
```

---

## Documents à mettre à jour

Si cette orientation est retenue :

* `docs/01-holocode/PARADIGME-HOLOSCENIQUE.md` : Introduire le concept de Nœud Fractal et de seuil d'échelle (`scale_threshold`).
* `docs/02-gouvernance/DECISIONS.md` : Ouvrir un ADR sur le budget mémoire matériel garanti et le sas d'imposteurs.
* `docs/04-roadmap/ROADMAP.md` : Intégrer le prototype de traversée fractale avant le réseau multi-joueur.
