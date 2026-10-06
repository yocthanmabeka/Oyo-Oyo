# Réponse de Gemini — les refus de HoloCode (2026-10-06)

Réponse au prompt [`../prompts/2026-10-06-gemini-refus.md`](../prompts/2026-10-06-gemini-refus.md), collée par Yocthan le 2026-10-06. Gemini n'a pas accès au dépôt. Résumé fidèle ; la lecture de Claude est à la fin. Le document jugé : [`proposals/Claude/pourquoi-ces-refus-2026-10/`](../../../proposals/Claude/pourquoi-ces-refus-2026-10/README.md).

## 1. Refus justes, refus de trop

**Justes** :

- **`div`** : comme Flutter, SwiftUI ou Jetpack Compose, qui n'en ont pas ; cela préserve le sens pour la vue à plat et pour la scène en profondeur.
- **Modifier la page à la main** : React, Flutter, Elm l'ont banni au profit d'un flux de données à sens unique ; l'autoriser détruirait l'arbitre et la synchronisation à plusieurs.
- **La cascade et les sélecteurs composés** : le web a passé quinze ans à la contourner (BEM, CSS Modules, Tailwind, CSS-in-JS).

**De trop** :

- **S'arrêter à `H3`** : une documentation technique, des conditions générales de vente, un article d'encyclopédie (1.2.3.1) deviennent impossibles sans casser le plan ou mettre du faux titre en gras.
- **Aucune superposition** : impossible d'afficher le nombre d'articles sur l'icône du panier, une pastille « En promo » sur une image, un bouton « Revenir en haut » flottant.
- **Aucun calcul sans alternative immédiate** : attendre des modules WebAssembly pour ajouter des frais de port ou filtrer une liste laisse un vide.

## 2. Les repères pour lecteurs d'écran

WCAG 2.2, critères 1.3.1 (information et relations) et 2.4.1 (contourner des blocs). Les aveugles sautent de repère en repère (touche D avec NVDA ou JAWS, le rotor avec VoiceOver) : `banner` (en-tête), `navigation`, `main`, `contentinfo` (pied), `complementary` (encadré).

**Gemini préfère des blocs** (`Header`, `Nav`, `Main`, `Footer`) à un rôle donné à un morceau (`Part(role: nav)`) :

1. un non-programmeur comprend `Nav(…)` ; `role:` est une abstraction technique ;
2. un bloc `Nav` a un sens dans l'espace : le moteur peut le poser sur une barre ou un socle d'orientation dans un monde.

## 3. Interdire le code libre : ce qu'ont fait les autres

- **Roblox (Luau)** : du code permis, mais dans une boîte hermétique ; flexibilité et succès, au prix d'une lutte permanente contre la triche, les scripts malveillants et les boucles infinies.
- **Second Life (LSL)** : des milliers de scripts en boucle ralentissaient les régions ; il a fallu des quotas de temps de calcul.
- **AMP (Google)** : JavaScript d'auteur interdit, composants tout faits ; pages rapides et stables, mais rejet des développeurs pour sa rigidité, puis abandon progressif.
- **Les e-mails HTML** : JavaScript banni partout ; des milliards d'e-mails par jour, mais une mise en page riche reste très difficile.

Conclusion de Gemini : bloquer le code libre protège le moteur et les téléphones, mais sans calcul déclaratif (arithmétique, filtres de listes), la plateforme restera faite de documents passifs.

## 4. Le survol sans la cascade

Trois façons comparées :

- **A — des états dans le style** : `.card { background: …; hover: { background: … } focus: { … } }`. Pas de sélecteur imbriqué ; l'état appartient au style.
- **B — des réglages sur le bloc** : `Button(…, hover_color: …)`. Mêle contenu et apparence.
- **C — un signal** : `On(Button.hover, effect: …)`. Verbeux pour un simple effet visuel.

**Recommandation de Gemini : A.**

## 5. Le refus manquant

**Les tailles en pixels pour le texte.** Avec le texte réglé à 150 % dans Android ou iOS, une page en `px` coupe le texte ou ignore le réglage. Gemini propose d'interdire les pixels pour le corps du texte, ou d'ajouter une unité relative (`rem`, ou un pourcentage du réglage de lecture du téléphone).

## Le tableau de Gemini

| Refus | Verdict | Pourquoi |
|---|---|---|
| `div` | Garder | `Row`, `Column`, `Grid` suffisent et gardent le sens pour la 3D |
| `section`, `article` | Assouplir | Admettre des blocs `Header`, `Nav`, `Main`, `Footer` |
| `h4` à `h6` | Assouplir | Aller jusqu'à `H6`, avec la même logique |
| `script` | Garder | Sécurité et moteur déterministe |
| Modifier la page à la main | Garder | Tout passe par l'arbitre |
| `display`, `position` dans un style | Assouplir | Garder l'interdit dans les styles, ajouter un bloc de superposition (`Stack`, `Badge`) |
| Cascade, sélecteurs composés | Garder | Les états sont des sous-réglages du style |
| `requestAnimationFrame` | Sans objet | Le moteur dessine |

---

## Lecture de Claude

**D'accord** :

- Sur les trois refus à garder (`div`, page modifiée à la main, cascade) et sur `script` : c'est aussi l'avis de Codex dans ses revues précédentes.
- **Le survol, option A.** Elle va avec la décision de Yocthan d'écrire les styles comme du CSS (`ADR-017`) : un état est un sous-bloc du style, pas un sélecteur.
- **Un bloc de superposition** : c'est ce que Claude proposait aussi (un badge, une barre fixe).
- **Les tailles relatives** : Claude l'avait classé en priorité dans le tableau ; Gemini a raison d'en faire un défaut du web que HoloCode répète.
- **Des blocs plutôt qu'un rôle pour les repères.** L'argument du non-programmeur est bon, et meilleur que celui de Claude. Claude change d'avis : `Nav`, `Header`, `Footer` comme blocs. À vérifier contre la règle des noms (`ADR-016`) et l'envie de Yocthan d'un langage peu verbeux.

**À nuancer** :

- **« Aucun calcul »** : ce n'est pas exact. `add`, `sub`, `set`, `random`, et le total d'un panier calculé seul (`Prices`) existent. Mais Gemini a raison sur le fond : des frais de port (« 5 euros sous 300 euros ») ou un filtre de liste sont impossibles aujourd'hui.
- **`H6`** : Gemini veut aller jusqu'à `H6` tout de suite ; Claude proposait `H4` au premier vrai besoin. Les deux tiennent ; c'est à Yocthan.
- L'option C de Gemini (`is_over ? navy : blue`) n'est pas une écriture HoloCode ; elle illustre l'idée.
- Les jugements sur Roblox, Second Life et AMP sont des résumés de Gemini ; Claude ne les a pas recoupés, mais ils correspondent à ce qui est largement connu (quotas de temps de script de Second Life, abandon de l'obligation AMP par Google en 2021).

**Ce qui reste** : l'avis de Codex sur le même prompt, puis la décision de Yocthan.
