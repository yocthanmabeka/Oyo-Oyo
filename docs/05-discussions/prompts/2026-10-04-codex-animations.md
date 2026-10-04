Bonjour Codex. C'est Yocthan. J'ai besoin de ton avis sur les animations et les passages dans le moteur de HoloCode. Tu as accès au dépôt `yocthanmabeka/Metaverse` : lis le code, lance-le, et réponds en français, en phrases simples. Ne fusionne rien et ne change aucun statut de décision : présente tes propositions dans une pull request ou dans `proposals/GPT5.6/`.

# Ce que tu dois regarder

1. Le fichier `moteur/web/page.html` : c'est lui qui anime les pages. Les fonctions à lire : `emettre`, `appliquer`, `entrerDans`, `fondu`, `franchir`, `ouvrirCarrefour`, `zoomer`, `entrerEnPoints`.
2. Le fichier `moteur/src/navigation.rs` : l'entrée dans un point du Big Bang (`zoomer`, `entrer`, `sortir`, `sprites`).
3. Le journal du 2026-10-04, dans `docs/06-journal/JOURNAL.md` : il raconte ce que j'ai dit et ce que Claude a changé.
4. Pour lancer : `moteur/README.md`. Puis ouvre `http://localhost:8080/exemples/boutique-comparee/boutique.holo`, `…/exemples/maison/salon.holo` et `…/mondes/big-bang.holo`.

# Ce qui existe aujourd'hui

| Geste | Ce qui se passe |
|---|---|
| Appuyer sur un bouton qui mène à un autre site (`On(Open.tap, effect: Workshop.enter)`) | Un fondu bref (un tiers de seconde), comme un lien du web |
| Toucher le point lumineux lui-même | Le point s'ouvre où il est, grandit jusqu'à remplir la fenêtre, devient le site |
| Zoomer dans une page qui a `points:` | Elle grossit jusqu'à 4 fois, puis ses pixels deviennent des points |
| Zoomer dans un point du Big Bang | Le point grossit jusqu'à déborder de l'écran, son monde grandit dedans, puis on est dedans |
| Le bouton « Carrefour » | Des portails ronds vers les mondes voisins |
| Un point qui mène au fichier d'un autre serveur | Passage obligé par le carrefour, qui affiche le nom du serveur |

Ma règle : par défaut, ce qui ressemble au web se comporte comme le web. L'effet de métavers est réservé aux gestes faits sur un objet du métavers (un point, le zoom). Un mouvement part toujours de ce qu'on a touché.

# Ce que je te demande

1. **Ton avis sur chaque passage du tableau.** Est-il clair pour quelqu'un qui connaît le web ? Est-il cohérent avec ma règle ? Dis ce que tu changerais, et pourquoi.
2. **Ta proposition pour les animations par défaut.** Durée, forme (fondu, glissement, zoom), et ce qui doit se passer avec les animations réduites. Compare avec ce que font les navigateurs (les View Transitions), Android, iOS et les jeux.
3. **Faut-il que l'auteur puisse choisir l'animation dans son fichier `.holo` ?** Si oui, propose l'écriture la plus petite possible, en comparant les options, et vérifie qu'on ne répète pas un défaut de CSS (`transition`, `animation`, `@keyframes`).
4. **Les défauts que tu trouves dans le code** de ces passages : cas où deux animations se chevauchent, clics pendant une animation, bouton « retour » du navigateur, téléphone.
5. **Le bouton unique.** Les outils du moteur (Vue points, Carrefour, Tourner, De face) sont maintenant rangés derrière un seul bouton rond, en bas à droite. Dis si c'est le bon endroit et la bonne forme, sur téléphone comme sur ordinateur.

Pour chaque point : sépare ce que tu as vérifié en lançant le code de ce que tu supposes. Termine par une liste classée, du plus important au moins important.
