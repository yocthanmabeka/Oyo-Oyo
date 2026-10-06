# ADR-022 — Deux façons d'aller ailleurs : le lien `A`, et le point qu'on traverse

- Statut : ACCEPTÉ — noms et écriture tranchés le 2026-10-06 (`ADR-047`)
- Date : 2026-10-03
- Responsable : Yocthan Mabeka
- Discussions sources : journal du 2026-10-03
- Validation : principe posé par Yocthan le 2026-10-03 ; il a demandé pourquoi le lien ne s'écrivait pas `A`, comme en HTML.
- Projets affectés : HoloCode, HoloEngine

## Contexte

Yocthan : « Lorsque l'on quitte la maison, on traverse la porte du salon et on va au jardin. On ne prend pas de véhicule : on utilise les mêmes jambes. » Il ne veut ni rechargement ni redirection entre les mondes : seulement l'animation et le zoom. Sauf pour un vrai changement de site « à l'ancienne », par un lien normal. Il a aussi relevé que le dézoom doit faire ressortir d'un monde, et demandé comment s'écrivent les listes de liens, HTML ayant `a`, `ul`, `ol`, `li`.

## Décision

1. **Le lien classique s'écrit `A`** : `A("texte", to: "adresse")`. On quitte la page pour une autre adresse, comme `<a href>`.
2. **Le passage se fait par un `Point`.** Son monde peut être écrit sur place (`inside: World(...)`) ou être un autre fichier (`inside: "garden.holo"`). On le traverse sans changer de page : carrefour, portail, animation. L'adresse du navigateur devient celle de l'autre fichier ; le bouton « retour » ramène.
3. **Dézoomer fait ressortir** : quand la page est déjà à sa taille normale, un cran de dézoom ramène au site ou au fichier d'où l'on venait.
4. **Les fichiers voisins sont lus d'avance.** Un `.holo` pèse quelques kilo-octets : lire ceux où mènent les points d'une page ne coûte presque rien, et rend le passage immédiat. Un fichier introuvable ou refusé laisse le passage fermé.
5. **Les listes gardent un seul bloc** : `List`, avec `ordered: true` pour la numéroter. Pas de `UL`, `OL` ni `LI`.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Le mot du lien | `A`, `Link` | `A`. C'est le mot de HTML, comme `P` et `H1` (`ADR-020`). Et en HTML, `link` désigne autre chose (le rattachement d'une feuille de style) : un programmeur du web lirait `Link` de travers (`ADR-016`, règle 2). |
| Le mot du passage | un bloc `Door`, ou `Point` | `Point`. Un point est déjà ce dans quoi l'on entre ; un mot de plus n'apporterait rien. « Porte » était une image de Yocthan. |
| Les listes | `UL`, `OL`, `LI` comme en HTML, ou `List` | `List`. Trois balises pour une liste est un défaut de HTML : `li` hors de `ul` ne veut rien dire, et l'on oublie de les refermer. |

Défauts de HTML, CSS et JavaScript évités : une adresse `javascript:` dans un lien (refusée : `to` n'accepte qu'un fichier voisin, un site de la page, ou `http`/`https`) ; le rechargement complet à chaque lien ; les applications qui évitent ce rechargement au prix d'un routeur écrit en JavaScript.

## Conséquences

### Positives

- Un ensemble de fichiers `.holo` forme un espace où l'on circule sans coupure, et chaque lieu garde une adresse.
- Aucun code à écrire pour l'auteur : le passage est une propriété du langage.

### Négatives et risques

- Le premier passage vers un autre serveur dépend du réseau ; la lecture d'avance le masque sans le supprimer.
- La limite `Zoom(levels:)` ne compte pas les passages d'un fichier à l'autre : on peut tourner en rond entre deux fichiers. C'est voulu : le salon mène au jardin, qui ramène au salon, comme on tourne dans une maison. Ce qui devait être borné, c'est la mémoire : le moteur ne garde que les 32 derniers fichiers lus.
- Vers un autre serveur, la barre d'adresse ne peut pas montrer l'adresse réelle (règle de sécurité des navigateurs) : elle affiche le fichier de départ suivi de `#@` et de l'adresse réelle. Dans un navigateur propre à l'Holoverse, cette gêne disparaîtrait.

## Complément du même jour : le site de quelqu'un d'autre

Les deux limites de la première version ont été revues à la demande de Yocthan.

- **« Seuls les fichiers rangés à côté » était une mauvaise limite** : sans elle levée, chacun reste enfermé dans son dossier, et il n'y a pas de web. `inside` accepte maintenant l'adresse complète d'un fichier `.holo`, en `http` ou `https`. Trois précautions : le portail affiche le nom du serveur ; un fichier d'un autre serveur n'est lu qu'à l'ouverture du carrefour, jamais d'avance (lire d'avance préviendrait ce serveur de chaque visite, sans que le visiteur ait rien demandé) ; et il passe par le même vérificateur que les autres, donc ne peut contenir aucun code.
- **« Rien ne borne le nombre de passages » était une bonne limite à moitié** : ne pas borner les passages est juste ; ne pas borner la mémoire ne l'était pas. Voir ci-dessus.

## Corrections après la revue de Codex (2026-10-04)

Codex a montré que plusieurs promesses de cette fiche ne tenaient pas (`proposals/GPT5.6/revue-langage-securite-2026-10-03/`). Corrigé :

- **Aucun contact sans geste.** Une adresse en `#@…` lisait le fichier distant dès l'ouverture, et le carrefour lisait tous les fichiers distants visibles. Désormais : un clic, un serveur, un fichier. Une adresse distante ouverte directement propose le passage au lieu de le faire.
- **L'origine reste visible.** Un bandeau du moteur, « Vous êtes chez … », reste affiché tant qu'on est dans le fichier d'un autre serveur.
- **En `https`.** Le `http` n'est accepté que vers sa propre machine, quand on y est déjà : une page publique ne peut pas faire partir de requêtes vers le réseau privé du visiteur.
- **Des limites de lecture.** 256 Ko par fichier, 8 secondes d'attente, sans cookies et sans dire d'où l'on vient.

Non corrigé, et dit : le poids déclaré (`weight`) n'est toujours pas comparé au poids réel ; il n'y a pas d'en-tête de sécurité `Content-Security-Policy` sur le serveur de démonstration.

## Correction du 2026-10-04 : on entre directement

Yocthan, en essayant la boutique : en appuyant sur « Enter the workshop », il arrivait sur le carrefour au lieu d'arriver dans l'atelier. « Quelqu'un qui est déjà habitué au web ne va pas trouver ça normal. » Le carrefour sert quand le visiteur veut choisir parmi d'autres mondes ; il ne doit pas s'interposer.

- Entrer dans un point mène maintenant directement au site. Par un bouton (`enter` demandé par une règle), c'est un passage ordinaire, comme sur le web : un fondu bref. En touchant le point lui-même, il s'ouvre sur place, grandit jusqu'à remplir la fenêtre, et devient le site. Yocthan, après un premier essai où le bouton déclenchait l'ouverture du point : « Il y a certains endroits où ça doit fonctionner comme le web normal, et d'autres où ça doit fonctionner comme le métavers. Là, tu chamboules tout. »
- Le carrefour ne s'ouvre que sur demande : son bouton, ou la capacité `portals`.
- Exception gardée : vers le fichier d'un autre serveur, le carrefour reste le passage obligé, parce qu'il affiche le nom du serveur et attend un clic (« aucun contact sans geste », ci-dessus).

## Critères de validation

- `exemples/maison/` : du salon au jardin par un point, sans rechargement ; retour par un dézoom.
- Tests du moteur : `plat.rs` (liens, refus des adresses dangereuses, passage vers un fichier).

## Conditions de réexamen

- Quand un navigateur propre à l'Holoverse permettra d'afficher la vraie adresse d'un fichier d'un autre serveur.
