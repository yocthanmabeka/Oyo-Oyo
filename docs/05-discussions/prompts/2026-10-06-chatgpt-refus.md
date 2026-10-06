# Prompt pour ChatGPT (conversation simple) — les refus de HoloCode (2026-10-06)

À copier en entier dans une conversation ChatGPT ordinaire. Le quota de Codex est épuisé : ChatGPT remplace ici Codex pour **un avis**, sans travail sur le dépôt. Tout ce qu'il faut savoir est dans ce texte.

---

**Consigne importante : ne travaille pas.** N'écris pas de code, ne lance pas de mode agent, ne cherche pas à ouvrir un dépôt, ne produis pas de fichier. On te demande seulement **une comparaison et un jugement de nécessité**, en français, en phrases simples, et vite. Sois franc : dis quand tu n'es pas d'accord.

## Le projet en dix lignes

Holoverse veut réinventer le web en métavers léger. Un fichier `.holo`, écrit en **HoloCode**, est un site web normal par défaut. Quand on zoome, les pixels deviennent des points, et chaque point contient un monde où l'on entre. Il doit tourner sur un téléphone ordinaire, et l'on doit pouvoir créer **sans programmer**. Le moteur est écrit en Rust, compilé en WebAssembly ; il fabrique le HTML et le CSS de la page. L'auteur n'écrit jamais de HTML, de CSS ni de JavaScript.

Exemple de HoloCode :

```
Page(
  title: "Ma boutique",
  state: State(panier: 0),
  children: [
    H1("Ma boutique"),
    Row(gap: 16px, children: [
      Text("{panier} au panier"),
      Button(name: Ajouter, text: "+"),
    ]),
  ],
  rules: [ On(Ajouter.tap, effect: panier.add(1)) ],
)
.carte { background: #fffaf3; border-radius: 14px; }
```

Les blocs (avec une majuscule) disent ce qu'ils sont ; les styles s'écrivent comme du CSS simple ; ce qui arrive s'écrit en règles visibles (`On`, `Every`, `When`), jamais en code libre.

## Ce qui a été fait avant ce prompt

1. **Un grand tableau** de 129 éléments de HTML, CSS et JavaScript : HoloCode en couvre environ **43 %** (HTML 47 %, CSS 47 %, JavaScript 32 %, estimations de Claude). Les manques jugés prioritaires : envoyer un formulaire, zone de texte, liste déroulante, boutons radio, listes et répétition, tableaux, vidéo, survol, tailles de texte qui s'adaptent, repères pour lecteurs d'écran, langue de la page, description de page pour les moteurs de recherche.
2. **Le pourquoi de huit refus**, écrit par Claude (l'IA qui programme le projet).
3. **L'avis de Gemini** sur ces refus.
4. **Une recherche sur ce que disent les humains** (enquêtes, forums, experts), avec une règle fixée par Yocthan, le responsable du projet : *si la majorité des humains montre que l'élément manque ou le défend, on lève le refus ; si elle en dit surtout du mal, on garde le refus.*

## Les huit refus, leurs raisons, et les avis déjà donnés

| # | Élément refusé | Pourquoi HoloCode le refuse | Qui l'a décidé | Ce que disent les humains | Gemini | Claude aujourd'hui |
|---|---|---|---|---|---|---|
| 1 | `div` | Des boîtes sans sens : lecteurs d'écran, moteurs de recherche et vue 3D ne savent plus ce qui est un titre ou un bouton. HoloCode a `Row`, `Column`, `Grid`, `Text`. | Yocthan (fiche acceptée) | Surtout du mal (« soupe de `div` » ; la spécification HTML l'appelle « dernier recours ») ; un expert rappelle qu'un `div` simple conteneur n'est pas mauvais en soi | Garder | Garder |
| 2 | `section`, `article` | Le plan est déjà donné par les titres `H1` à `H3`. | Claude seul, sans décision | Grande confusion depuis 2008 | Assouplir : des blocs `Header`, `Nav`, `Main`, `Footer` | Garder `section`/`article`, mais **admettre** `Nav`, `Header`, `Footer`, `Main` en blocs |
| 3 | `h4`, `h5`, `h6` | Éviter de choisir un titre « parce qu'il est plus petit » ; le numéro dit la place dans le plan, jamais la taille. La fiche dit : « on en ajoutera si un vrai besoin apparaît ». | Yocthan | Rares sur un site, mais utiles aux longs documents (technique, juridique) ; 71,6 % des aveugles naviguent par les titres | Assouplir jusqu'à `H6` | Admettre |
| 4 | `script` (code libre) | Sécurité (on entre dans les mondes d'inconnus), vérification avant affichage, même résultat sur deux téléphones pour le jeu à plusieurs. Le calcul viendra par des fonctions pures et des modules WebAssembly enfermés. | Yocthan | **Forte plainte** contre l'interdiction : AMP (sans JavaScript) abandonné par la plupart des grands éditeurs dès 2021 ; constructeurs de sites critiqués pour leurs murs ; e-mails sans JavaScript pénibles à mettre en page. Mais Second Life a payé le code libre en lenteurs, Roblox enferme le code. | Garder | Pas de code libre ; avancer le **code enfermé** (limites de temps et de mémoire) et des règles plus riches (listes, calcul, « sinon ») |
| 5 | Modifier la page à la main (DOM) | Le « code spaghetti » ; ici seul le moteur change la page, à partir des valeurs et des règles. | Yocthan | Surtout du mal ; React 44,7 % contre jQuery 23,4 % (Stack Overflow 2025) | Garder | Garder |
| 6 | `display`, `position`, `float`, `z-index` dans un style | La disposition vient des blocs ; une mise en page faite à la main casse sur téléphone et n'a pas de sens dans un monde 3D. | Yocthan | « Évitez `position: absolute` dans 99 % des cas » pour la mise en page ; indispensable pour un badge, une bulle, un menu ; l'ancrage est la nouveauté CSS la plus aimée (State of CSS 2026) | Assouplir : un bloc de superposition (`Stack`, `Badge`) | Garder dans les styles ; ajouter un bloc de superposition |
| 7 | Sélecteurs composés, cascade, `!important` | Un style ajouté ici en casse un autre ailleurs ; HoloCode vise par type de bloc (`P { }`) ou par nom (`.carte { }`), un seul nom par bloc. | Yocthan | Surtout du mal (« guerres de spécificité ») ; Tailwind, qui évite la cascade, est le cadre le plus utilisé (environ 37 %) mais divise ; des experts défendent la cascade bien utilisée | Garder ; les états (survol, focus) en sous-blocs du style : `.carte { hover: { … } }` | Garder ; ajouter les états comme le propose Gemini |
| 8 | `requestAnimationFrame` | Sans objet : le moteur dessine. | — | — | Sans objet | Sans objet |

**Refus manquant**, trouvé par Gemini : HoloCode n'a que des **tailles en pixels**. Le réglage « texte plus grand » du visiteur est alors ignoré (environ un adulte sur trois de plus de 65 ans grossit le texte). Tout le monde s'accorde : il faut des unités relatives.

## Ce qu'on te demande

Réponds en moins de 900 mots :

1. **Pour chacun des huit refus**, ton verdict : *garder*, *assouplir* ou *supprimer*, et la raison en une phrase. Dis si tu es d'accord avec Claude, avec Gemini, ou avec aucun des deux.
2. **Les trois désaccords** à trancher :
   - les repères : des blocs (`Nav`, `Header`, `Footer`, `Main`) ou un rôle donné à un morceau (`Part(role: nav)`) ?
   - les titres : `H4` seulement, au premier besoin, ou tout de suite jusqu'à `H6` ?
   - le code : comment répondre à la plainte des humains contre l'interdiction, sans ouvrir la porte au code libre dans un monde d'inconnus ?
3. **La règle de Yocthan** (« on suit la majorité des humains ») : est-elle bonne pour un langage destiné à des **non-programmeurs**, quand ceux qui parlent sur les forums sont surtout des **développeurs** ? Que proposerais-tu de mieux, en une phrase ?
4. **Le plus nécessaire** : parmi tout ce qui manque à HoloCode (la liste du point 1 de « Ce qui a été fait »), les **cinq** choses à faire en premier, dans l'ordre, avec une phrase pour chacune.

Termine par **un seul tableau** : élément, ton verdict, ta raison.
