# ADR-050 — Les composants, faits pour le web

- Statut : ACCEPTÉ ; `Part` renommé `Component` le 2026-10-07 (`ADR-056`), qui ajoute aussi les valeurs par défaut et les signaux émis
- Date : 2026-10-06
- Responsable : Yocthan Mabeka
- Discussions sources : `docs/01-holocode/COMPARATIF-LANGAGES-WEB.md` (les composants, note la plus basse de HoloCode : 45) ; la demande de Yocthan du 2026-10-06 ; les prompts envoyés à Gemini et Codex le même jour (`docs/05-discussions/prompts/2026-10-06-*-composants-et-comparatif.md`), dont les réponses sont encore attendues
- Validation : Yocthan, le 2026-10-06 : « J'aime le composant. C'est une notion de Flutter que j'adore […] il faudra des composants faits vraiment pour le web […] grâce au CSS, tu vas l'utiliser facilement » ; puis « je suis tes propositions, je les valide, tu as mon feu vert ».
- Projets affectés : HoloCode, HoloEngine

## Contexte

Un morceau (`Part`, `ADR-029`) n'avait ni paramètres, ni règles : on ne pouvait pas écrire une carte d'article une fois et la poser dix fois avec un titre et un prix différents. Les widgets de Flutter le font, mais changer l'allure d'un widget demande de passer des paramètres de style à la main ou de toucher au thème. Le web, lui, restyle de l'extérieur par le CSS. Yocthan veut les deux : l'écriture de Flutter, la souplesse du CSS.

## Décision

1. **Un composant s'écrit une fois** : `Part(name: ArticleCard, params: [title, price], children: [ … ], rules: [ … ])`, dans `parts: [ … ]` d'une page, ou à la racine d'un fichier importé.
2. **Il se pose comme un bloc**, à la manière d'un widget Flutter : `ArticleCard(name: Sunrise, title: "Sunrise", price: 120)`. Tous les paramètres sont donnés et nommés ; une faute est refusée avec le bon mot.
3. **Un seul bloc racine**, comme le widget que rend Flutter. `Use(Menu)` reste pour un morceau sans paramètres ni règles, qui peut avoir plusieurs blocs.
4. **Un paramètre s'emploie par son nom** : `{title}` dans un texte (avec format : `{price:cents}`), `title` à la place d'une valeur. **Donné par le nom d'une valeur de la page** (`qty: sunrise`), il la suit : `{qty}` la montre, `qty.add(1)` la change. C'est ainsi qu'un composant agit sur la page, sans arbitre nouveau.
5. **Les blocs nommés reçoivent le nom de la copie** (`Add` → `AddSunrise`), comme dans `Repeat` ; les règles du composant sont écrites une fois par copie, et rejoignent celles de la page, du monde, ou de la répétition qui le contient.
6. **Le restylage se fait par le CSS, de l'extérieur** :
   - `ArticleCard { … }` vise toutes les copies ;
   - `ArticleCard.promo(…)` puis `.promo { … }` vise une copie ;
   - une variable employée dans le composant (`--accent`) peut être redéfinie par un composant ou un nom de style (`.promo { --accent: crimson; }`), pour ce bloc et son contenu.
7. **Plusieurs noms de style par bloc** (`P.card.big(…)`, quatre au plus) : nécessaire pour qu'une copie garde le style de son composant et reçoive le sien. Un nom de style s'écrit en minuscules ; un nom qui commence par une majuscule est réservé à la marque d'un composant.
8. **Déplié à la lecture**, comme `Use` et `Repeat` : le reste du moteur ne voit que des blocs ordinaires. Le HTML produit est du vrai HTML : la racine de chaque copie porte `holo-c-ArticleCard` et `holo-s-promo` ; le CSS est du vrai CSS.
9. **Garde-fous** : seize paramètres au plus ; un composant ne se pose jamais lui-même ; huit niveaux de composants au plus ; deux mille copies par page au plus ; un paramètre ne porte ni un mot du langage ni le nom d'une valeur de la page.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Poser une copie | `Use(ArticleCard, title: …)` ; **`ArticleCard(title: …)`** | Écrire le composant comme un bloc est l'écriture de Flutter, que Yocthan maîtrise ; un composant devient un mot de son propre langage. |
| Déclarer les paramètres | `props` (React, Vue) ; `input` (Angular) ; **`params`** | « params » se lit sans connaître un framework ; `props` est un jargon de React. |
| Restyler | paramètres de style (Flutter) ; `::part()` (Web Components) ; **le nom du composant, des noms de style, des variables** | Les trois s'écrivent avec ce que le débutant connaît déjà (`P { }`, `.card { }`, `--or`) ; aucun sélecteur composé, aucun mot nouveau. |
| Agir sur la page | un signal du composant (`ArticleCard.add`) ; **un paramètre qui reçoit le nom d'une valeur** | Le second ne demande rien de nouveau à l'arbitre, et se lit comme une phrase : « la quantité, c'est sunrise ». |
| Un emplacement pour du contenu (`children`, `<slot>`) | maintenant ; **plus tard** | Utile pour une fenêtre ou une carte à contenu libre ; à ajouter quand un exemple réel le demandera. |
| Des valeurs propres à chaque copie | maintenant ; **par un paramètre** | Une copie reçoit le nom de sa valeur (`qty: sunrise`) : la page garde toutes les valeurs, l'arbitre reste simple. |

## Conséquences

- La leçon 70 (`exemples/lecons/70-composants.holo`), le guide (§ 6 sexies bis), `NOMS.md`.
- Une valeur de la page est nécessaire par copie qui compte quelque chose ; une liste à champs (point 2 du plan) l'allégera.
- Les réponses de Gemini et de Codex au prompt du 2026-10-06 seront lues : si elles montrent un défaut, on corrige avant que d'autres pages ne s'en servent.

## Correction du 2026-10-06, après la revue de Codex (PR 124)

Codex a relevé que les noms de style d'un fichier importé (`.card`) valent pour toute la page, et que si deux fichiers importés écrivent le même style, le premier gagnait en silence. Il propose de cacher automatiquement les noms de style d'un composant importé, comme Vue ou Svelte.

Décidé : **deux fichiers importés qui écrivent le même style sont refusés**, avec leurs deux noms ; la page peut toujours réécrire un style importé. Les noms de style d'un fichier importé **restent partagés** avec la page : le site de référence s'en sert comme d'un thème commun (`.carte`, `.prix` dans `commun.holo`), et les cacher le casserait. Un composant qui ne veut rien partager se style par son nom, `ArticleCard { … }`, qui ne vise que ses copies. Si un vrai composant importé a besoin de styles cachés, on y reviendra avec la proposition de Codex.

## Critères de validation

- Tests du moteur : une copie posée comme un bloc, ses classes, ses règles par copie, son restylage par le nom du composant, un nom de style et une variable ; un composant dans une répétition et dans un fichier importé ; les refus (paramètre oublié, mal écrit, donné deux fois, composant qui se pose lui-même, nom de bloc du langage, plusieurs racines, copies sans nom).
- La leçon 70 s'ouvre dans Chrome : trois cartes, la troisième d'une autre couleur, le panier qui se remplit.
