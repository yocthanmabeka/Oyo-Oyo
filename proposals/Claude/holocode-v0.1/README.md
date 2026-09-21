# Proposition code by Claude — HoloCode v0.1

## Contribution

- Sujet : noyau exécutable du paradigme holoscénique, avec lois, capacités, composition d'archétypes et relations à plusieurs paires
- Discussions sources : `HC-002`, `HC-003`, `HC-004`, et la revue critique `HC-012` (fiche proposée, pas encore fusionnée)
- Décisions concernées : `ADR-003`, `ADR-004`, `ADR-005`
- Statut proposé : `EXPÉRIMENTATION`
- Implémentation : Python 3.11 ou plus récent, sans dépendance ; dossier autonome qui ne modifie aucun document existant

Cette proposition est indépendante de la « Proposition code by GPT5.6 » (PR n° 1). Le protocole prévoit que deux IA traitent le même problème séparément pour comparer leurs résultats.

## Hypothèse testée

> Un langage qui connaît les espaces, les unités, les relations, les lois et les capacités peut refuser avant l'exécution des erreurs qu'un programme orienté objet laisse passer, et expliquer après coup chaque changement d'état sans code supplémentaire.

Le prototype ne cherche pas à prouver que HoloCode est plus rapide ou plus court que la POO. Il cherche à montrer ce que le langage sait, et ce qu'il en fait.

## Le programme d'exemple

Extrait de [exemples/maison.holo](exemples/maison.holo) :

```holocode
archetype Openable {
    state { opened: Bool = false }
    capability open  { opened = true }
    capability close { opened = false }
}

world Home {
    space House {
        entity Alice: Person at (0m, 0m, 0m)
        entity BackDoor: Openable + Lockable at (20m, 0m, 1m) { locked = true }
    }
    space Garden {
        entity Gate: Openable at (0m, 0m, 1m)
    }

    relation Near(p: Person, d: Openable) when distance(p, d) <= 2m

    law LockedStaysClosed {
        forall d: Openable + Lockable where d.locked
        forbid d.open
    }

    phenomenon AutoClosing {
        forall d: Openable
        when d.opened and no Near(_, d) for 3s
        effect d.close
    }
}
```

## Définitions proposées

Ces définitions répondent à quatre points de la dette décisionnelle du registre. Ce sont des propositions.

| Notion | Définition v0.1 |
|---|---|
| **Archétype** | Un lot de champs et de capacités. Il n'y a ni héritage ni priorité. |
| **Composition** | `A + B` réunit les archétypes à plat. Deux archétypes qui déclarent le même champ ou la même capacité ne se composent pas : c'est une erreur, pas une surcharge. |
| **Capacité** | Seul moyen de modifier un état. Une capacité n'écrit que dans les champs de son propre archétype. |
| **Relation** | Un ensemble de paires d'entités, recalculé à chaque pas. `Near` n'est pas un booléen : `some Near(_, d)` demande s'il existe une paire dont `d` est le second membre. |
| **Espace** | Un repère. Deux entités d'espaces différents ne sont jamais comparées, même si leurs coordonnées sont identiques. |
| **Loi** | Une règle permanente qui ne change jamais l'état : elle interdit une capacité tant qu'une condition tient. Elle vaut pour toute entité composée des archétypes cités, et pour tout demandeur, phénomène ou joueur. |
| **Phénomène** | Une transformation : quand sa condition tient pour une entité, il demande une capacité. Il ne peut rien écrire directement. |
| **Temps** | `for 3s` signifie « vrai sans interruption depuis 3 secondes ». La durée est comptée en secondes simulées et ne dépend pas de la taille du pas. |

La frontière entre loi et phénomène devient nette : **un phénomène propose, une loi dispose**. Une loi l'emporte toujours sur un phénomène. Deux phénomènes qui se contredisent ne sont pas départagés : le pas s'arrête.

## Sémantique d'un pas

`tick(dt)` se déroule toujours dans cet ordre :

1. le temps avance de `dt` ;
2. les relations sont recalculées sur un instantané stable, espace par espace ;
3. chaque phénomène examine chaque entité concernée, dans l'ordre de déclaration, et formule des intentions ;
4. les lois refusent les intentions interdites ;
5. si deux intentions autorisées écrivent des valeurs différentes dans le même champ, le pas est annulé en entier et une erreur nomme les deux phénomènes ;
6. les capacités autorisées s'appliquent ;
7. chaque changement et chaque nouveau refus entre dans le journal causal.

Une invocation externe (`world.invoke("BackDoor", "open")`) suit les étapes 4 à 7. Il n'existe aucune autre façon d'écrire un état : `entity.state` est en lecture seule.

## Ce que le journal causal produit

Sortie réelle de `python exemples/demo_maison.py` :

```text
[t=1 s, pas 1] FrontDoor.open par le phénomène AutoOpening
    opened : false -> true
    parce que some Near(_, FrontDoor) : Near(Alice, FrontDoor) à 1.00 m
    parce que not FrontDoor.opened
[t=1 s, pas 1] BackDoor.open REFUSÉ par la loi LockedStaysClosed (demandé par le phénomène AutoOpening)
    parce que BackDoor.locked
[t=6 s, pas 6] FrontDoor.close par le phénomène AutoClosing
    opened : true -> false
    parce que FrontDoor.opened
    parce que no Near(_, FrontDoor) depuis 3 s (exigé : 3 s)
```

`world.why("BackDoor", "opened")` retourne le dernier événement qui a changé ce champ.

## Ce que le vérificateur refuse avant l'exécution

[exemples/erreurs.holo](exemples/erreurs.holo) contient six erreurs, toutes signalées avec leur ligne et leur colonne, sans exécuter une seule seconde de simulation :

- une coordonnée écrite en secondes ;
- une distance de relation écrite en secondes ;
- une longueur comparée à une durée (`d.width > 3s`) ;
- une capacité qui écrit un champ d'un autre archétype ;
- une composition de deux archétypes qui déclarent le même champ ;
- un phénomène qui demande une capacité que ses archétypes n'offrent pas.

Le vérificateur émet aussi un avertissement quand deux phénomènes peuvent écrire des valeurs différentes dans le même champ de la même entité et que leurs conditions ne s'excluent pas. `AutoOpening` (`some Near`) et `AutoClosing` (`no Near`) s'excluent : aucun avertissement.

## Correspondance avec le critère de réussite de la vision

| Critère de [VISION.md](../../../docs/00-vision/VISION.md) | Dans ce prototype |
|---|---|
| Un monde et deux espaces | `Home`, avec `House` et `Garden` |
| Plusieurs entités composées | `FrontDoor: Openable + Lockable` |
| Une relation spatiale typée | `Near(p: Person, d: Openable)`, distance en mètres |
| Une loi indépendante des entités | `LockedStaysClosed` |
| Un phénomène déclenché de manière déterministe | `AutoOpening`, `AutoClosing` ; deux exécutions donnent le même journal |
| Une trace expliquant pourquoi l'état a changé | le journal causal et `world.why` |

## Comparaison avec la proposition GPT5.6

Comparaison faite à la lecture de la PR n° 1, sans l'exécuter.

| Point | GPT5.6 v0.1 | Claude v0.1 |
|---|---|---|
| Entités | Une par nom, sans archétype | Archétypes composables, autant d'entités que voulu |
| Relation | Un booléen entre deux entités nommées | Un ensemble de paires, interrogé par `some` et `no` |
| Phénomène | Porte sur des entités nommées | Quantifié : `forall d: Openable` |
| Lois | Absentes | Présentes, prioritaires sur les phénomènes |
| Écriture de l'état | `effect Door.opened = true`, directe | Par capacité uniquement |
| Unités | Mètre | Longueurs et durées, vérifiées avant l'exécution |
| Temps | Nombre de ticks | Secondes simulées, `for 3s` |
| Espaces | Absents | Plusieurs, jamais comparés entre eux |
| Conflits | Erreur à l'exécution | Avertissement à la vérification, puis erreur à l'exécution, pas annulé en entier |
| Journal | Changements effectifs | Changements, refus et raisons |
| Calcul des relations | Une comparaison par relation | Grille spatiale |
| Vérification | Mêlée au runtime | Phase séparée, toutes les erreurs en une passe |

La proposition de GPT est plus courte et plus simple à lire. La mienne va plus loin sur la sémantique, et elle est donc plus exposée aux objections ci-dessous.

## Comparaison avec l'existant

Le protocole demande de comparer toute primitive aux solutions existantes. Aucune primitive de ce prototype n'est nouvelle.

| Primitive | Précédent | Ce que fait le prototype |
|---|---|---|
| Archétype composé | ECS (Unity DOTS, Bevy, Flecs) pour les données ; traits de Rust pour le comportement | Proche d'un ECS pour la composition des champs, avec refus des collisions à la vérification. Différent pour le comportement : dans un ECS, l'archétype est une donnée passive et le comportement vit dans des systèmes ; ici, les capacités sont attachées à l'archétype |
| Relation interrogeable | Datalog, relations de Flecs | Limitée à la proximité entre deux entités |
| Phénomène | Règles de production, règles événement-condition-action | Identique, quantifié sur un archétype |
| Loi | Contraintes, invariants ; règles « Instead » d'Inform 7 | Limitée à l'interdiction d'une capacité |
| Capacité | Encapsulation : un mutateur confiné à son archétype | Seul accès en écriture à l'état. Ce n'est pas le modèle object-capability : une capacité au sens de la sécurité est un jeton infalsifiable et transférable, et ce prototype n'en a pas. Le mot vient du paradigme du dépôt |
| Unités | F# | Deux dimensions : longueur et durée |
| `for 3s` | Langages synchrones, logique temporelle | Un seul opérateur |
| Journal causal | Event sourcing | Produit par le runtime, sans code dans le programme |

Les lignes « Archétype composé », « Loi » et « Capacité » ont été corrigées après la revue comparative de Gemini : la première version disait à tort « identique à l'ECS » et rattachait les capacités au modèle object-capability.

L'apport éventuel n'est donc pas dans les briques. Il est dans leur réunion en une seule sémantique où le vérificateur connaît à la fois les unités, les archétypes, les capacités et les règles. Ce prototype montre que cette réunion est faisable en petit. Il ne montre pas qu'elle vaut mieux qu'un ECS bien conçu.

## Objections et limites

- **C'est un interpréteur Python.** Il ne dit rien des performances sur téléphone ni du budget de 1 Go. Aucun chiffre de vitesse n'est revendiqué.
- **La grille spatiale est reconstruite à chaque pas.** Le test montre qu'elle évite la double boucle (moins de 1 % des comparaisons sur 500 × 500 entités espacées), pas qu'elle tient un budget fixe. Des entités entassées dans une même case ramènent au coût quadratique.
- **Un conflit entre phénomènes arrête le monde.** C'est déterministe et compréhensible, mais ce n'est pas un arbitrage. Un monde partagé ne peut pas s'arrêter : il faudra des priorités ou des portées, à concevoir.
- **L'analyse des conflits est syntaxique.** Elle reconnaît `some` contre `no` et `x` contre `not x`. Elle ne raisonne pas sur les valeurs : `d.width > 2m` et `d.width < 1m` ne sont pas reconnus comme exclusifs. Elle peut donc avertir à tort. Elle n'examine que les phénomènes entre eux, sur les entités déclarées.
- **Une loi ne sait qu'interdire une capacité.** Une loi physique continue (gravité, propagation) n'est pas exprimable.
- **Une règle ne quantifie qu'une variable.** « Pour toute paire (personne, porte) » n'est pas exprimable ; les capacités n'acceptent que des constantes ; il n'y a ni expression, ni arithmétique.
- **Les entités sont fixes.** Ni création, ni destruction, ni changement d'espace pendant l'exécution. La position est une entrée du monde (`world.move`), pas un état protégé.
- **L'autorité est une convention d'API Python, pas une barrière de sécurité.** Un programme hôte malveillant peut contourner `state`. Une vraie garantie demande un runtime isolé.
- **Pas de HoloIR**, pas de persistance, pas de réseau, pas de rendu.

## Expérience ou preuve requise

1. Écrire le même scénario en POO et avec un ECS de référence (Flecs), puis compter pour chaque version : les lignes, les six erreurs du contre-exemple détectées avant exécution, et le code nécessaire pour obtenir l'équivalent du journal causal.
2. Soumettre à ce prototype et à celui de GPT dix programmes et dix contre-exemples communs (phase 1 de la feuille de route), écrits par une troisième partie.
3. Remplacer l'arrêt sur conflit par une règle d'arbitrage, et vérifier qu'elle reste explicable dans le journal.
4. Mesurer la grille sur des entités entassées et en mouvement, pour savoir si un budget fixe par pas est tenable.

## Lancer

Depuis ce dossier :

```bash
python -m unittest discover -s tests -v          # 27 tests
python exemples/demo_maison.py                   # scénario commenté
python -m holocode exemples/maison.holo --ticks 5
python -m holocode exemples/erreurs.holo --check # six erreurs attendues
```

## Grammaire

```ebnf
programme   = { archetype } monde { archetype } ;
archetype   = "archetype" NOM "{" { etat | capacite } "}" ;
etat        = "state" "{" { NOM ":" TYPE "=" valeur } "}" ;
capacite    = "capability" NOM "{" { NOM "=" valeur } "}" ;
monde       = "world" NOM "{" { espace | relation | loi | phenomene } "}" ;
espace      = "space" NOM "{" { entite } "}" ;
entite      = "entity" NOM ":" types "at" "(" valeur "," valeur "," valeur ")" [ "{" { NOM "=" valeur } "}" ] ;
types       = NOM { "+" NOM } ;
lieur       = NOM ":" types ;
relation    = "relation" NOM "(" lieur "," lieur ")" "when" "distance" "(" NOM "," NOM ")" ( "<" | "<=" ) valeur ;
loi         = "law" NOM "{" "forall" lieur [ "where" condition ] "forbid" NOM "." NOM "}" ;
phenomene   = "phenomenon" NOM "{" "forall" lieur "when" condition "effect" NOM "." NOM "}" ;
condition   = terme { "and" terme } ;
terme       = [ "not" ] atome [ "for" valeur ] ;
atome       = ( "some" | "no" ) NOM "(" argument "," argument ")"
            | NOM "." NOM [ COMPARAISON valeur ] ;
argument    = NOM | "_" ;
valeur      = "true" | "false" | TEXTE | NOMBRE [ UNITE ] ;
TYPE        = "Bool" | "Number" | "Text" | "Length" | "Duration" ;
UNITE       = "mm" | "cm" | "m" | "km" | "ms" | "s" | "min" | "h" ;
```

## Documents à mettre à jour

Aucun document existant n'est modifié par cette proposition. Si Yocthan la retient, en tout ou en partie :

- [PARADIGME-HOLOSCENIQUE.md](../../../docs/01-holocode/PARADIGME-HOLOSCENIQUE.md) : corriger l'exemple `AutomaticDoor` et reprendre la définition de la loi et du phénomène ;
- [DECISIONS.md](../../../docs/02-gouvernance/DECISIONS.md) : ouvrir des ADR sur la composition des archétypes, sur l'écriture par capacité et sur la priorité des lois ;
- [ROADMAP.md](../../../docs/04-roadmap/ROADMAP.md) : placer la comparaison avec la POO et l'ECS avant la construction de HoloIR.
