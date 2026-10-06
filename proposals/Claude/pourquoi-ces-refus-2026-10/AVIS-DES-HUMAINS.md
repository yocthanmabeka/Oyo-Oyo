# Ce qu'en disent les humains : les refus de HoloCode face aux avis du monde entier

## Contribution

- Auteur : Claude, à la demande de Yocthan, le 2026-10-06.
- Demande : vérifier sur Stack Overflow, Reddit, Twitter et les forums ce que les humains pensent des éléments que HoloCode refuse, et appliquer cette règle : **si la majorité des humains montre que l'élément leur manque ou défend son usage, on lève le refus ; si les humains en disent surtout du mal, on garde le refus.**
- Statut : **EXPLORATION**. La règle est celle de Yocthan ; son application ci-dessous est l'avis de Claude. Rien n'est décidé : on attend aussi Codex.
- Le document jugé : [`README.md`](README.md) (pourquoi chaque refus).

## Ce que cette recherche vaut, honnêtement

- **Ce n'est pas un vote mondial.** Personne ne peut compter « la majorité des humains ». Claude s'est appuyé sur trois sortes de sources, de la plus solide à la plus faible :
  1. **des enquêtes chiffrées** : State of CSS (4 902 réponses en 2026), WebAIM, l'enquête des utilisateurs de lecteurs d'écran (1 539 réponses, 2024), l'enquête Stack Overflow 2025 ;
  2. **des textes d'experts et la spécification HTML** ;
  3. **des discussions de développeurs** (Reddit, blogs, listes du W3C), souvent rapportées par des articles.
- **Twitter (X) n'a pas pu être fouillé directement** : ses messages sont mal indexés par les moteurs de recherche. Stack Overflow apparaît par son enquête annuelle.
- **Les gens qui écrivent sur les forums sont surtout des développeurs.** HoloCode vise d'abord des personnes qui ne programment pas. Quand les développeurs se plaignent d'une contrainte, cela ne dit pas forcément ce que vivra un débutant.

## Refus par refus

### 1. `div` → **garder le refus**

- **Ce que disent les humains** : la critique de la « soupe de `div` » est massive. Des pages faites de centaines de boîtes sans sens sont plus mal lues par les moteurs de recherche, plus difficiles à maintenir, et inaccessibles ; on y recrée à la main, en JavaScript, des boutons et des titres que le navigateur offrait déjà. La spécification HTML elle-même appelle `div` « l'élément de dernier recours ».
- **La voix contraire** : des experts de l'accessibilité (Scott O'Hara) rappellent qu'un `div` n'est pas mauvais en soi comme simple conteneur, et qu'on ne devrait pas chasser chaque `div` quand il ne gêne personne.
- **Selon la règle** : les humains en disent surtout du mal, et HoloCode a déjà les conteneurs qui ont un sens (`Row`, `Column`, `Grid`). **On garde.**

### 2. `section`, `article` → **garder le refus** ; mais les repères `nav`, `header`, `footer`, `main` → **les admettre**

- **Ce que disent les humains** : `section` et `article` sont parmi les balises les plus mal comprises depuis 2008. Des développeurs l'écrivent sur les listes du W3C : « elle n'apporte rien de plus qu'un `div`, sauf la confusion ». Il faut des tutoriels pour des éléments qui devraient être évidents, et beaucoup mettent des `section` sans titre.
- **Les repères, eux, servent vraiment.** Enquête WebAIM 2024 : **63 %** des utilisateurs de lecteurs d'écran se servent des repères au moins de temps en temps (17,9 % dès qu'ils existent, 13,9 % souvent, 31,5 % parfois), et cet usage **remonte** après dix ans de baisse.
- **Les titres comptent plus que tout** : **71,6 %** de ces utilisateurs explorent une page en sautant de titre en titre. C'est le premier moyen de lire une page, très loin devant les autres.
- **Selon la règle** : pour `section` et `article`, les humains sont surtout confus, **on garde le refus**. Pour les repères (`nav`, `header`, `footer`, `main`), une partie croissante des personnes aveugles en a besoin, **on lève le refus**. C'est aussi l'avis de Gemini (des blocs `Nav`, `Header`, `Footer`) et ce que Codex signalait comme manquant dès le 2026-10-03.

### 3. `h4`, `h5`, `h6` → **lever le refus**

- **Ce que disent les humains** : ces niveaux sont rarement utilisés sur un site ordinaire (trois niveaux suffisent le plus souvent). Mais personne n'en dit du mal : ils servent aux **documents longs et structurés**, comme les documentations techniques et les textes juridiques, et les systèmes de design recommandent d'en prévoir **au moins quatre**. Comme les aveugles naviguent d'abord par les titres (71,6 %), un document long avec un plan coupé à trois niveaux leur fait perdre le fil.
- **Selon la règle** : aucun avis négatif sur ces balises, et un besoin réel montré. **On lève le refus**, avec la même règle qu'aujourd'hui : le numéro dit la place dans le plan, jamais la taille.

### 4. `script` (le code libre) → **la règle dit d'assouplir** ; Claude recommande un code **enfermé**, pas un code libre

- **Ce que disent les humains** : c'est le point où les humains **contredisent le plus** la vision.
  - **AMP**, de Google, interdisait le JavaScript des auteurs. Les développeurs ont jugé les limites « draconiennes » ; dès que Google a retiré l'obligation en mai 2021, **la plupart des grands éditeurs (CNN, le Washington Post…) l'ont abandonné**.
  - Sur **Squarespace**, plus de **35 %** des avis citent le manque de personnalisation comme premier défaut ; les constructeurs de sites sans code sont critiqués pour leurs murs.
  - Dans **les e-mails**, où le JavaScript est interdit partout, la mise en page reste réputée pénible, avec des tableaux comme dans les années 1990.
- **La voix contraire** : là où le code est permis dans un monde partagé, il coûte cher. **Second Life** a dû ralentir le temps de ses régions et répartir le temps de calcul des scripts ; **Roblox** enferme le code dans une boîte et lutte en permanence contre la triche et les boucles infinies (résumé de Gemini, non recoupé par Claude).
- **Selon la règle** : les humains se plaignent nettement de l'absence de code. **Le refus strict est contredit.** Mais les plaintes viennent de développeurs, et la sécurité d'un métavers ouvert aux inconnus reste un vrai danger. **Recommandation de Claude** : ne pas autoriser le JavaScript libre, mais avancer la porte de sortie déjà prévue : des **fonctions de calcul** et des **modules enfermés** (`ADR-013`), avec des limites de temps et de mémoire, comme Roblox. Pour un débutant, ajouter d'abord ce qui manque le plus : des listes, des calculs (frais de port), un « sinon ».

### 5. Modifier la page à la main (le DOM) → **garder le refus**

- **Ce que disent les humains** : l'avis dominant est que la modification directe de la page, à la jQuery, mène au « code spaghetti » et aux bugs quand plusieurs parties du code changent la page en même temps. Tout l'écosystème moderne (React, Vue, Svelte, Flutter) décrit la page à partir de l'état au lieu de la modifier pas à pas.
- **Les chiffres** : enquête Stack Overflow 2025 : React est utilisé par **44,7 %** des développeurs, jQuery par **23,4 %**. jQuery reste donc très présent, surtout dans le code ancien, mais l'admiration est du côté déclaratif. (Nuance : l'admiration pour React elle-même a baissé, de 62 % à 52 %.)
- **Selon la règle** : les humains en disent surtout du mal. **On garde.**

### 6. `display`, `position`, `float`, `z-index` dans un style → **garder le refus pour la mise en page** ; **ajouter un bloc de superposition**

- **Ce que disent les humains** : le consensus est très clair et coupé en deux. **Pour la mise en page** : « évitez `position: absolute` dans 99 % des cas, utilisez Flexbox ou Grid » ; un élément positionné sort du flux et chevauche les autres. **Pour les badges, les bulles d'aide, les menus déroulants et les fenêtres** : il est indispensable.
- **Les chiffres** : dans State of CSS 2026, la fonction nouvelle la plus aimée est l'**ancrage** (*Anchor Positioning*), qui sert justement à poser un élément sur un autre.
- **Selon la règle** : pour la mise en page, les humains en disent du mal, **on garde le refus**. Pour la superposition, ils montrent un vrai besoin, **on l'admet sous forme de bloc** (un badge, une bulle, une barre fixe), comme le proposent Gemini et Claude.

### 7. Sélecteurs composés, cascade, `!important` → **garder le refus**

- **Ce que disent les humains** : `!important` est l'un des sujets les plus critiqués du CSS : les « guerres de spécificité », où chacun surenchérit jusqu'à ce que personne ne sache plus d'où vient un style. Dans State of CSS 2026, la **spécificité** et la **gestion de la cascade** figurent parmi les douleurs citées, avec la surcharge mentale d'un CSS jugé trop complexe.
- **Le signe le plus fort** : **Tailwind**, qui sert justement à éviter la cascade, est le cadre CSS le plus utilisé (environ **37 %** des répondants de State of CSS 2025, selon les sites qui citent l'enquête).
- **La voix contraire** : Tailwind divise. Ses critiques le trouvent laid, proche des styles en ligne des années 2000 ; une discussion Reddit de près de 900 votes le dit. Et des experts reconnus (Andy Bell, CUBE CSS) défendent la cascade comme une force, bien utilisée.
- **Selon la règle** : les humains disent surtout du mal de la cascade incontrôlée et de `!important`. **On garde**, et l'on ajoute les états (survol, focus) dans le style, comme le propose Gemini.

### 8. Le refus manquant, trouvé par Gemini : le texte en pixels → **l'ajouter aux manques urgents**

- **Ce que disent les humains** : quand les tailles de texte sont en pixels seulement, le réglage de taille du texte du visiteur est **ignoré**. Environ **un adulte sur trois de plus de 65 ans** grossit le texte de son navigateur, et pour les malvoyants, c'est le premier outil. Le consensus est net : des unités relatives (`rem`).
- **Selon la règle** : un avis largement partagé contre ce que fait HoloCode aujourd'hui. **À corriger** : des tailles de texte qui suivent le réglage du visiteur.

## Le résultat en un tableau

| Élément | Ce que disent surtout les humains | Selon la règle de Yocthan | Recommandation de Claude |
|---|---|---|---|
| `div` | Du mal (la « soupe de `div` ») | Garder le refus | Garder |
| `section`, `article` | De la confusion | Garder le refus | Garder |
| `nav`, `header`, `footer`, `main` | Un besoin croissant des aveugles (63 %) | Lever le refus | Admettre, en blocs |
| `h4` à `h6` | Rares, mais nécessaires aux longs documents | Lever le refus | Admettre |
| `script` | Une forte plainte contre l'interdiction (AMP abandonné) | Assouplir | Pas de code libre ; du code **enfermé** et des règles plus riches |
| Modifier la page à la main | Du mal (le code spaghetti) | Garder le refus | Garder |
| `position` pour la mise en page | Du mal | Garder le refus | Garder |
| `position` pour un badge, une bulle | Un besoin | Lever le refus | Admettre, en bloc de superposition |
| Cascade, `!important` | Du mal | Garder le refus | Garder ; ajouter les états dans le style |
| Texte en pixels | Du mal | À corriger | Unités relatives |

## Sources

- [La spécification HTML et la soupe de `div` (Scott O'Hara, « Divisive »)](https://www.scottohara.me/blog/2022/01/20/divisive.html) · [Pourquoi le HTML sémantique compte](https://www.evoluted.net/blog/development/why-semantic-html-matters-more-than-ever) · [« Be lazier with semantic HTML »](https://dev.to/wtaylor45/be-lazier-with-semantic-html-3mge)
- [WebAIM, enquête des utilisateurs de lecteurs d'écran n° 10 (2024)](https://webaim.org/blog/screen-reader-user-survey-10-results/) · [Résumé d'Allyant](https://allyant.com/webaim-screen-reader-survey-10-key-takeaways-for-2024-and-beyond/)
- [La confusion `section` / `article` (SitePoint)](https://www.sitepoint.com/better-take-the-webs-temperature-its-coming-down-with-another-itis/) · [Liste WHATWG, 2008](https://lists.w3.org/Archives/Public/public-whatwg-archive/2008Jul/0268.html)
- [Les niveaux de titres dans un système de design (Uxcel)](https://app.uxcel.com/lessons/typographic-hierarchy-107/size-of-your-smallest-heading-3127) · [Les titres H1 à H6](https://cse.sc.edu/~oreillyj/kd/h.html)
- [AMP en 2020, « mieux mais pas bien »](https://areknawo.com/google-amp-in-2020-better-but-still-not-good/) · [Les éditeurs et la fin de l'obligation AMP (Search Engine Land)](https://searchengineland.com/will-publishers-drop-amp-when-its-no-longer-a-requirement-for-top-stories-335612) · [Press Gazette](https://pressgazette.co.uk/google-amp-publishers-top-stories) · [Forum Publii : JavaScript refusé dans AMP](https://forum.getpublii.com/topic/google-amp-issue-custom-javascript-not-allowed/)
- [Squarespace, 200 avis analysés](https://www.twicecommerce.com/blog/squarespace-pros-and-cons?hsLang=de) · [Avis Capterra](https://www.capterra.com/p/143461/Squarespace/reviews/6166256)
- [E-mails HTML : tableaux et pas de JavaScript (Mailtrap)](https://mailtrap.io/blog/html-email-best-practices/)
- [Second Life : dilatation du temps et scripts](https://wiki.secondlife.com:443/wiki/LlGetRegionTimeDilation) · [Liste des développeurs de Second Life](https://list-archives.secondlife.com/opensource-dev/2010-March/000681.html)
- [jQuery et code spaghetti face au déclaratif](https://medium.com/@echelamoses/react-vs-vanilla-javascript-why-you-should-stop-writing-dom-manipulation-code-dfd2d08d4b4a) · [Enquête Stack Overflow 2025 : React et Next.js](https://pagepro.co/blog/react-tldr/stack-overflow-2025-react-and-next-js-grow-in-use-drop-in-admiration/)
- [Moins de `position: absolute` en CSS moderne (Ahmad Shadeed)](https://ishadeed.com/article/less-absolute-positioning-modern-css/) · [Quand utiliser `position: absolute`](https://builderio.mimo.org/glossary/css/position-absolute)
- [State of CSS 2026, les douleurs](https://2026.stateofcss.com/en-US/pain-points/) · [Patrick Brosset sur State of CSS 2026](https://patrickbrosset.com/articles/2026-08-07-state-of-css-2026/) · [State of CSS 2025, autres outils](https://2025.stateofcss.com/th-TH/other-tools) · [Frameworks CSS en 2025 (Tailkits)](https://tailkits.com/blog/popular-css-frameworks/)
- [Les dangers de `!important`](https://dev.to/timonwa/the-dangers-of-using-important-in-css-why-it-should-be-avoided-4gcd) · [La guerre de la spécificité](https://vivianvoss.net/blog/the-specificity-war)
- [Le débat Tailwind (The New Stack)](https://thenewstack.io/tailwind-css-debate-another-cool-tool-dissed-by-web-purists/) · [Builder.io](https://www.builder.io/blog/the-tailwind-css-drama-your-users-don%27t-care-about) · [CUBE CSS, Andy Bell (Smashing)](https://smashingmagazine.com/2020/06/smashing-podcast-episode-19)
- [Pixels ou `rem` pour l'accessibilité (Engage)](https://engageinteractive.co.uk/blog/em-vs-rem-vs-px) · [OSnews](https://www.osnews.com/story/138624/the-surprising-truth-about-pixels-and-accessibility/) · [GitLab : passer aux `rem`](https://gitlab.com/gitlab-org/gitlab/-/issues/28896)
