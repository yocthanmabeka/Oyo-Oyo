# Proposition by Gemini — v0.1 : HoloFractal Core

## Contribution

- **Sujet :** Noyau spatial fractal, transition d'échelle continue, garantie d'empreinte mémoire bornée ($O(1)$ théorique < 1 Go) et ontologie unifiée Nœud = Avatar = Monde.
- **Discussions sources :** `HC-001`, `HC-005`, `HC-007`, `HC-013`
- **Décisions concernées :** `ADR-003`, `ADR-005`, `ADR-006`, `ADR-007`
- **Statut proposé :** `EXPÉRIMENTATION`
- **Implémentation :** Python 3.11 ou plus récent, bibliothèque standard stricte, sans dépendance externe.

---

## Avertissement méthodologique

Ce prototype vérifie une **logique de comptage et de gestion de quotas déclarés en mémoire abstraite**, et non une mesure physique d'occupation VRAM/RAM sur silicium mobile réel. Les coûts mémoire sont calculés à partir de représentations quantifiées (sommets compressés 16 octets, textures ASTC, structures d'état) et d'imposteurs de taille fixe (64 Ko). La validation physique finale relève d'un démonstrateur WebGPU exécuté sur smartphone.

---

## Hypothèse testée

> Si l'on unifie l'entité, le monde et le portail sous une primitive unique (le Nœud Fractal) et que l'on applique une règle de chargement à la demande avec congélation en imposteurs au franchissement des seuils d'échelle, il est possible de naviguer dans une arborescence de mondes imbriqués sans dépasser un budget mémoire déclaré (< 1 Go), avec un coût mémoire actif borné à chaque niveau.

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
   * **Quotas individuels par nœud :** Chaque nœud déclare un plafond de triangles (`max_triangles`) et de mémoire (`max_memory_bytes`), strictement vérifié à l'enregistrement.
   * **Cohérence des mutations d'état :** Toute addition ou suppression de variable d'état réajuste en temps réel le solde du registre pour prévenir toute dérive ou valeur négative.
4. **Réactivité Déclarative Sans Code :** Les interactions s'associent par liaisons Cause $\to$ Effet (Signaux $\to$ Actions) déterministes sans boucles libres, incluant la mutation d'état, le morphing d'apparence et la propagation d'impulsions (`PULSE_SIGNAL`).

---

## Comparaison avec les propositions existantes

| Critère | GPT-5.6 (PR #1) | Claude (PR #2) | Gemini (v0.1) |
|---|---|---|---|
| Forme du Monde | Boîte 3D fermée | Salles isolées (Espaces) | Arborescence fractale continue |
| Gestion du 1 Go | Aucune | Aucune | Comptabilité stricte au niveau de l'octet + Sas d'imposteurs |
| Modèle d'accès | DSL textuel déclaratif (relations & phénomènes) | DSL déclaratif avec vérificateur de types & lois | Nœuds déclaratifs réactifs (Zéro-Code ready) |
| Passage à l'échelle | Dégradation globale | Grille spatiale locale | Paging hiérarchique : mémoire active locale bornée |

---

## Objections et limites

1. **Calcul théorique d'octets déclarés :** En Python pur, les coûts sont calculés sur des modèles théoriques et non sur des allocations de mémoire physique du système d'exploitation mobile.
2. **Perte de parallaxe de l'imposteur :** Quand le monde parent et les frères sont congelés en imposteurs cubiques, ils sont perçus sous forme de skybox/proxy simplifié, nécessitant un traitement graphique adapté pour masquer la transition.
3. **Non-Turing complétude volontaire :** L'interdiction des boucles libres et du code arbitraire empêche les algorithmes procéduraux complexes en cours d'exécution, au bénéfice de la stabilité du runtime sur mobile.

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
