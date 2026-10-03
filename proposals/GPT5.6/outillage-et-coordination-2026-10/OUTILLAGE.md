# Outillage vérifié

`MCP` est un protocole qui permet à une IA d'utiliser un outil. « Gratuit » signifie ici que le logiciel ne demande pas de licence ; les services cloud, crédits, réseau et matériel peuvent rester payants.

| Outil | Mainteneur et réalité | Gratuit / prérequis Windows 11 | Droits et risques | Verdict |
|---|---|---|---|---|
| [Chrome DevTools MCP](https://github.com/ChromeDevTools/chrome-devtools-mcp) | Équipe ChromeDevTools/Google ; pilote et inspecte Chrome, traces de performance, réseau, console, captures | Apache-2.0 ; Node LTS, npm, Chrome stable | Lit et modifie le profil Chrome piloté ; télémétrie active par défaut ; accès réseau. Utiliser un profil vide et couper statistiques/CrUX | **Indispensable à la demande** pour diagnostiquer, pas comme service permanent |
| [Playwright MCP](https://github.com/microsoft/playwright-mcp) | Microsoft ; automatise le navigateur via l'arbre d'accessibilité | Apache-2.0 ; Node 18+ ; télécharge des navigateurs si absents | Contrôle le navigateur et les pages. Le CLI est recommandé par Microsoft aux agents de code car plus économe en contexte | **Tests Playwright indispensables ; MCP optionnel** |
| [Android Platform Tools / ADB](https://developer.android.com/tools/releases/platform-tools) | Google ; connexion USB, commandes et redirection du protocole Chrome | Gratuit ; archive légère, pilote USB Samsung parfois nécessaire | Le téléphone affiche une autorisation de débogage ; ADB peut fortement contrôler l'appareil | **Indispensable maintenant** pour la preuve sur le Z Flip 5 |
| [Débogage Chrome Android](https://developer.chrome.com/docs/devtools/remote-debugging) | Google ; `chrome://inspect`, USB, inspection d'un onglet réel | Inclus avec Chrome + ADB | USB debugging et confirmation sur le téléphone | **Indispensable maintenant**, sans MCP supplémentaire |
| [Mobile MCP](https://github.com/mobile-next/mobile-mcp) | Mobile Next, communautaire ; Android/iOS, appareils réels et émulateurs | Open source ; Node/npx, ADB pour Android | Capture écran, touche, saisie, lancement d'apps ; télémétrie par défaut ; problèmes Windows ont été rapportés | **Utile plus tard** pour gestes automatisés ; inutile pour la première mesure manuelle |
| [Blender MCP](https://github.com/ahujasid/mcp-for-blender) | Communautaire, explicitement non affilié à Blender Foundation | Open source ; Blender, Python/`uv`; configuration d'un module Blender | Contrôle Blender et ouvre un serveur local ; Blender est lourd | **Inutile maintenant** : le projet génère formes/textures par formules |
| [Figma MCP](https://help.figma.com/hc/en-us/articles/32132100833559-Guide-to-the-Dev-Mode-MCP-Server) | Figma ; donne le contexte d'un design aux agents | Conditions et accès liés au compte/siège Figma ; détails tarifaires à revérifier au moment du choix | Accès aux fichiers Figma autorisés | **Utile plus tard** si une maquette Figma devient la source ; sinon inutile |
| [Context7](https://github.com/upstash/context7) | Upstash ; fournit aux IA une documentation de bibliothèques à jour | Serveur open source ; service public avec limites/clé optionnelle selon l'offre | Requêtes et extraits de code envoyés au service | **Optionnel** ; les docs officielles et le dépôt suffisent aujourd'hui |
| [ElevenLabs MCP](https://github.com/elevenlabs/elevenlabs-mcp) | ElevenLabs officiel, mais le dépôt annonce ne plus être activement maintenu ; génération/transcription audio | Clé API, crédits (niveau gratuit annoncé), Python/`uv` | Envoi de texte/audio au cloud, lecture/écriture dans un dossier configuré | **Inutile maintenant** ; préférer Web Audio et son procédural |
| MCP de génération d'images | Aucun produit précis n'était nommé | **Non vérifié** | Dépend du fournisseur et d'une clé | **Ne pas installer** sans besoin et fournisseur déterminés |
| Pont MCP vers Gemini/ChatGPT | Aucun pont officiel précis n'était nommé | **Non vérifié** | Risque de clés payantes, boucles et attribution floue | **À rejeter** : GitHub est plus simple et traçable |
| [Gemini Code Assist for GitHub](https://developers.google.com/gemini-code-assist) | Google ; relit les PR et répond à `/gemini` dans les commentaires | Offre exacte/quota à confirmer sur le compte lors de l'installation | Application GitHub avec accès choisi au dépôt privé | **Meilleur moyen d'inclure Gemini**, sous réserve de l'écran de permissions |
| [Syntaxe VS Code](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide) | Une petite extension propre au projet, fondée sur une grammaire TextMate | Gratuit ; VS Code/Node seulement pour la construire | Aucun droit spécial si elle ne fait que colorer | **À construire bientôt**, sans MCP |
| Validateur `.holo` | À dériver du parseur/vérificateur existant du dépôt | Gratuit ; Rust déjà présent | Lecture seule des fichiers du dépôt | **Indispensable bientôt** : une commande unique utilisée par VS Code et la CI |

## Commandes exactes, à exécuter seulement après accord

Ces commandes **configurent** le client MCP ; elles ne sont pas exécutées dans cette PR.

### Chrome DevTools MCP

Claude Code :

```powershell
claude mcp add chrome-devtools -- npx -y chrome-devtools-mcp@latest --no-usage-statistics --no-performance-crux
```

Codex :

```powershell
codex mcp add chrome-devtools -- npx -y chrome-devtools-mcp@latest --no-usage-statistics --no-performance-crux
```

Le mode `--slim --headless` économise du contexte et l'affichage, mais ne remplace pas le mode complet pour les traces de performance.

### Playwright MCP — seulement si le MCP est réellement nécessaire

```powershell
claude mcp add playwright npx @playwright/mcp@latest
codex mcp add playwright npx "@playwright/mcp@latest"
```

### Mobile MCP — différé

```powershell
claude mcp add mobile-mcp -- npx -y @mobilenext/mobile-mcp@latest
codex mcp add mobile-mcp npx "@mobilenext/mobile-mcp@latest"
```

Avant usage, définir `MOBILEMCP_DISABLE_TELEMETRY=1`. Ne jamais lancer son mode HTTP sur `0.0.0.0` sans authentification.

### Documentation OpenAI — option distante, lecture seule

```powershell
claude mcp add --transport http openaiDeveloperDocs https://developers.openai.com/mcp
codex mcp add openaiDeveloperDocs --url https://developers.openai.com/mcp
```

Source officielle : [OpenAI Docs MCP](https://developers.openai.com/learn/docs-mcp).

### Outils différés

Je ne donne pas de commande Figma, Blender, Context7 ou ElevenLabs : ils ne sont pas retenus maintenant et certaines installations/conditions changent. Copier une commande non nécessaire augmenterait le risque d'installation accidentelle. Les pages officielles liées ci-dessus restent la source au moment d'une décision.
