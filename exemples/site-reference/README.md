# L'atelier des mondes — le site de référence

Construit par Claude le 2026-10-06 d'après le cahier des charges de Codex ([PR 79](https://github.com/yocthanmabeka/Metaverse/pull/79), `proposals/GPT5.6/site-reference-2026-10-06/`). Tout est écrit en HoloCode : aucune ligne de HTML, de CSS ou de JavaScript écrite par l'auteur.

| Page | Fichier | Ce qu'elle met à l'épreuve |
|---|---|---|
| Accueil | [`accueil.holo`](accueil.holo) | page témoin, ordinaire, sans moteur au départ |
| Catalogue | [`catalogue.holo`](catalogue.holo) | douze créations, `Grid`, images décrites |
| Fiche | [`fiche.holo`](fiche.holo) | texte long, titres, liste, citation |
| Panier et atelier | [`panier.holo`](panier.holo) | `State`, `Prices`, `keep`, `Input`, `Checkbox`, `If` ; l'atelier (`Point`, `World`), un pixel planté, points, relief, carrefour |
| Jardin | [`jardin.holo`](jardin.holo) | le lieu voisin, dans un autre fichier |
| Journal | [`journal.holo`](journal.holo) | article long, `Code`, liens |
| Disponibilité | [`disponibilite.holo`](disponibilite.holo) | `Data` depuis `stock.json` toutes les 5 s |
| Jeu | [`jeu.holo`](jeu.holo) | `Board`, `Every`, `When`, `meets`, `within`, `drag`, `Key`, `Sound`, pause, meilleur score gardé |
| Galerie | [`galerie.holo`](galerie.holo) | `Enter`, `Loop`, `Scenes`, `letters`, `each` |
| Menu et pied de page | [`commun.holo`](commun.holo), [`pied.holo`](pied.holo) | `Part`, `Use`, `import` |

Pour l'ouvrir : `node moteur/outils/server.mjs`, puis `http://localhost:8080/exemples/site-reference/accueil.holo`.

La recette de Codex n'est **pas** encore exécutée : ses résultats seront consignés à part. Ce qui a été vérifié à la construction est dans le journal du 2026-10-06.
