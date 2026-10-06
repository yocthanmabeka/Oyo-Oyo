# ADR-029 — Les imports : un morceau (`Part`) qu'on pose (`Use`)

- Statut : ACCEPTÉ
- Date : 2026-10-04
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-04 ; `docs/01-holocode/COMPARATIF-CONCURRENTS.md` (planning, étape 5) ; `ADR-013`, `ADR-016`
- Validation : Yocthan, le 2026-10-04, après le jeu de la pomme : « Après, tu vas continuer avec l'étape 5. » L'écriture est une proposition de Claude ; à juger après essai. Validé par Yocthan le 2026-10-06 : « Qu'est-ce que tu attends pour valider tous ceux qui sont à l'essai ? »
- Projets affectés : HoloCode, HoloEngine

## Contexte

Étape 5 du planning, première moitié. Un site de plusieurs pages a un menu et un thème communs. Sans import, on les recopie dans chaque fichier ; Gemini avait prévenu que c'était une dette dès les premiers sites. La directive `import "fichier.holo"` existait dans la grammaire depuis `ADR-013`, lue mais jamais appliquée.

## Décision (à l'essai)

Le fichier commun, `commun.holo` :

```holo
Part(
  name: Menu,
  children: [
    Row(children: [
      A("Home", to: "accueil.holo"),
      A("Contact", to: "contact.holo"),
    ]),
    Hr(),
  ],
)

Page { background: #f6f1e7; color: #2b2118; }
H1 { color: #8a3b12; }
```

Une page qui s'en sert :

```text
import "commun.holo"

Page(
  title: "The little studio",
  children: [
    Use(Menu),
    H1("Welcome"),
  ],
)
```

1. **Un fichier importé est un morceau** : `Part(name: Menu, children: [...])`, suivi de ses styles. Ce n'est pas une page.
2. **`import "commun.holo"`**, en haut de la page, le rend disponible. Le fichier est rangé à côté : ni adresse complète, ni remontée de dossier.
3. **`Use(Menu)`** pose les blocs du morceau à cet endroit, autant de fois qu'on veut.
4. **Les styles du morceau viennent avec lui.** Si la page écrit le même style, c'est le sien qui reste. C'est ainsi qu'un morceau sert aussi de thème commun.
5. Une fois les morceaux posés, tout se passe comme si la page avait été écrite d'un seul tenant : mêmes vérifications, mêmes règles. Un bouton du morceau a son nom, et une règle de la page peut l'écouter.
6. **Le moteur ne lit jamais un fichier tout seul.** C'est la page d'entrée, ou le moteur en ligne de commande, qui va chercher les fichiers importés et les joint au texte de la page.
7. Garde-fous : seize imports au plus ; un morceau n'importe pas d'autres fichiers ; un morceau n'a ni valeurs ni règles.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Ce qu'on importe | un morceau nommé, avec ses styles ; un fichier de styles à part et un fichier de blocs à part ; des composants avec paramètres | Un morceau avec ses styles. Un seul mécanisme sert au menu et au thème. Les composants à paramètres sont une affaire bien plus grosse (c'est le cœur de React) ; ils viendront si un vrai besoin le demande. |
| Comment le poser | `Use(Menu)` ; écrire `Menu()` comme un bloc du langage | `Use(Menu)`. Un nom de bloc reste un mot du langage : en lisant `Menu()`, on ne saurait pas s'il vient du langage ou d'un fichier. |
| Les mots | `Part`/`Use` ; `Component`/`Include` ; `Fragment`/`Insert` | `Part` et `Use`, courts et de tous les jours. `Fragment` est déjà pris (le morcellement), `Component` est du jargon. |
| Deux styles pour la même cible | une erreur ; le dernier écrit gagne ; la page gagne | La page gagne. Un thème commun qu'une page ajuste est le cas le plus courant ; l'erreur l'interdirait, et « le dernier écrit » est la cascade de CSS, qu'`ADR-017` refuse. |
| Qui lit les fichiers | le moteur ; celui qui l'appelle | Celui qui l'appelle. Le moteur reste une fonction pure de son texte : mêmes fichiers, même résultat, et rien à surveiller côté sécurité dans le cœur. |

Défauts du web évités : `<link>` et `<script src>` chargés depuis n'importe où ; l'ordre de chargement qui change le résultat ; un fichier CSS commun dont une règle écrase une page sans prévenir.

## Conséquences

### Positives

- Un site de plusieurs pages s'écrit sans recopier : `exemples/site/`.
- La page arrive toujours déjà fabriquée par le serveur, imports compris.

### Négatives et risques

- Une erreur dans un morceau est signalée à la ligne de l'`import`, avec le nom du fichier et la ligne dans ce fichier ; l'éditeur ne peut pas encore y sauter.
- Pas de paramètres : un morceau est le même partout. Le menu ne peut pas souligner « la page où l'on est ».
- Un morceau n'a ni valeurs ni règles : un panier commun à plusieurs pages n'est pas encore possible.
- Chaque import est une requête de plus au premier affichage dans le navigateur.
- **La seconde moitié de l'étape 5 n'est pas faite** : les données venues d'un serveur.

## Ce qui reste à faire

- Les données venues d'un serveur, par un pont surveillé.
- Des morceaux à paramètres, si un exemple réel le demande.
- `module` et `bridge` restent lus et non appliqués.

## Critères de validation

- `exemples/site/accueil.holo` et `contact.holo` : le même menu et le même thème, venus de `commun.holo` ; la page de contact garde sa propre couleur de titre.
- Tests du moteur : `holo.rs` (`un_fichier_importe_est_un_morceau_qu_on_pose`).

## Conditions de réexamen

- Quand Yocthan aura essayé et jugé : `import`, `Part`, `Use`.
- Au premier site qui demande un morceau à paramètres.
