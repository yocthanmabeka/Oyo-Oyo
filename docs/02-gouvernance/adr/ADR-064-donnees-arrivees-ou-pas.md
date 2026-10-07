# ADR-064 — Des données qui disent « arrivées » ou « échec »

- Statut : PROPOSITION (construit et essayé ; attend la validation de Yocthan)
- Date : 2026-10-07
- Responsable : Yocthan Mabeka
- Discussions sources : l'ordre de Yocthan du 2026-10-07 (« terminer les données : … chargement et erreur ») ; l'exploration de l'issue #82 (piste 1 : un échec de `Data` passe sans rien dire ; défaut D10 : une seule image refusée efface toute la liste).
- Validation : à faire par Yocthan.
- Projets affectés : HoloCode, HoloEngine

## Contexte

`Data(from: "stock.json")` (`ADR-030`) lisait un fichier à côté de la page. Quand il n'arrivait pas (pas de réseau, une erreur du serveur, un fichier trop gros ou illisible), rien ne se passait : la page ne pouvait ni dire « Chargement… », ni dire « échec », ni proposer de réessayer. Une lecture qui ne finissait jamais n'avait pas de fin.

Et une liste reçue dont une seule fiche avait une image hors du dossier (`../x.svg`, `javascript:…`) ou pas d'image disparaissait en entier, sans rien dire (D10).

## Décision

1. **Un nom** : `data: Data(name: Shop, from: "shop.json")`. Comme pour un module (`ADR-045`) ou un formulaire (`ADR-042`), le nom est celui d'un bloc, avec une majuscule.
2. **Deux signaux** que les règles écoutent :
   - `Shop.done` : les données sont arrivées et rangées dans les valeurs ;
   - `Shop.failed` : elles ne sont pas arrivées. C'est le cas sans réseau, sur une erreur du serveur, ou quand le fichier est trop gros (plus de 64 Ko), illisible (autre chose qu'un objet JSON) ou trop lent (plus de **10 secondes**).
3. **Une capacité** : `On(Retry.tap, effect: Shop.refresh)` relit les données.
   - **Une seule lecture à la fois** : pendant une lecture, la demande est sans objet, puisque la lecture en cours répondra.
   - **Une seconde au moins entre deux lectures** : une demande trop proche attend son tour. Elle n'est pas perdue : sinon « Chargement… » resterait affiché.
   - Une règle qui relit à chaque échec relit donc une fois par seconde au plus, comme `every: 1s`.
4. **« Chargement… »** n'est pas un mot nouveau. C'est une valeur de la page, à 1 au départ, que `Shop.done` et `Shop.failed` remettent à 0 (leçon 84).
5. **Sans nom**, rien ne change : la page lit ses données sans rien dire, comme avant.
6. **Une image reçue qu'on ne peut pas montrer** (hors du dossier, ou absente) ne retire que l'image. La ligne garde son texte de remplacement (`alt`). Un `phone:` ou un `alt:` qui manquent à la fiche sont laissés de côté.

## Comparaison faite avant de choisir

| Option | Écriture | Pour | Contre |
|---|---|---|---|
| **A. Des signaux, comme un module** (proposée) | `Data(name: Shop, …)`, `On(Shop.failed, effect: …)`, `Shop.refresh` | les mêmes mots que `Module` et `Form` ; l'auteur décide quoi montrer | « Chargement… » demande une valeur et deux règles |
| B. Un état tenu par le moteur | `Data(…, status: shopStatus)`, `If(shopStatus, is: "failed")` | moins de règles | un texte que le moteur change seul ; un mot de plus |
| C. Des blocs tout faits | `Data(…, loading: [ … ], failed: [ … ])` | court | des blocs rangés dans les données, pas dans la page |

Défauts du web évités : avec `fetch`, une erreur 404 n'est pas une erreur (il faut tester `response.ok`), et une demande sans fin n'a pas de délai (il faut un `AbortController`). Une boucle « réessayer à chaque échec » écrite à la main bombarde le serveur. Ici, l'échec couvre tous ces cas, le délai est de 10 secondes, et les relectures sont espacées d'office.

## Conséquences

- `GUIDE.md` (section 6 septies) et le tableau des limites disent les 10 secondes et la seconde entre deux lectures.
- Reste à faire, dans une autre pull request : la page fabriquée par le serveur avec ses données. Aujourd'hui, elle part des valeurs de départ, puis les données arrivent dans le navigateur.

## Critères de validation

- Tests du moteur : le nom, les signaux, la capacité et les refus (`server_data_goes_through_the_arbiter`) ; une image refusée ou absente ne retire que l'image (`a_list_with_fields_is_shown_filled_and_comes_from_the_server`).
- Dans Chrome (`browser-tests.mjs`, essai « Data ») :
  - les données arrivent ;
  - un fichier absent, illisible ou trop gros donne un échec ;
  - un fichier trop lent donne un échec après 10 secondes ;
  - deux demandes rapprochées sont servies l'une après l'autre ;
  - une demande pendant une lecture ne crée pas de lecture en trop ;
  - la leçon 84 montre ses nouvelles.
