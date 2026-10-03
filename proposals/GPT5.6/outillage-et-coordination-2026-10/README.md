# Outillage et coordination — proposition code by GPT5.6

## Contribution

- Auteur : GPT5.6 (Codex)
- Date : 2026-10-03
- Portée : outillage, lacunes du projet et coordination entre IA
- Statut demandé : **PROPOSITION** — aucune décision n'est modifiée

## Résultat proposé

Le dispositif le plus simple est **GitHub comme boîte aux lettres unique**, sans pont direct permanent entre IA. Claude, Codex et Gemini lisent les mêmes issues et pull requests (PR, propositions de modification). Les serveurs MCP servent seulement à donner un outil précis à une IA ; ils ne constituent pas un dialogue fiable entre modèles.

À installer maintenant, après accord de Yocthan :

1. **Aucun nouveau pont entre IA.** Le connecteur GitHub déjà disponible suffit à Codex ; Claude dispose déjà de `gh`.
2. **Android Platform Tools (ADB) et Chrome DevTools manuel** pour mesurer le Samsung Z Flip 5. C'est plus léger et plus fiable que d'installer immédiatement Mobile MCP.
3. **Playwright en dépendance de test du dépôt**, plus tard dans une PR d'implémentation, pour les scénarios reproductibles et les captures visuelles. Pour un agent de code, Microsoft indique que son CLI est plus économe en contexte que le MCP.
4. **Chrome DevTools MCP**, seulement quand l'IA doit diagnostiquer performances, console ou réseau. Il démarre Chrome à la première utilisation, pas au simple branchement.

Ne pas installer maintenant : Mobile MCP, Blender MCP, Figma MCP, ElevenLabs MCP, Context7 ou un générateur d'images MCP. Ils répondent à des besoins futurs, doublonnent un outil plus simple, consomment des ressources, ou demandent une clé/crédits.

## Objections et limites

- Les mesures de RAM n'ont pas été exécutées : aucun outil n'a été installé et le téléphone n'était pas relié à cette session. Les chiffres de RAM par outil seraient donc inventés ; la PR définit une méthode de mesure plutôt qu'une estimation.
- Un test visuel sur ordinateur ne prouve pas le comportement du Z Flip 5. La preuve finale reste un relevé sur l'appareil réel.
- Le MCP Chrome peut voir et modifier toutes les données du profil Chrome piloté. Il faut un profil de test vide.
- Mobile MCP est communautaire et contrôle réellement le téléphone ; il ne doit être activé que sur un appareil/profil de test.
- Gemini Code Assist peut relire les PR depuis GitHub, mais sa disponibilité et ses quotas dépendent du compte. Il faut vérifier l'écran d'installation avant de lui ouvrir le dépôt privé.

## Expérience ou preuve

Cette contribution s'appuie sur l'état de `main` lu le 2026-10-03 : moteur Rust/WASM présent, CI présente, 19 tests annoncés dans `AGENTS.md`, transfert initial annoncé à 502 Ko, mesures sur téléphone encore en attente. Je n'annonce pas avoir relancé ces tests dans cette contribution documentaire.

Preuve minimale à obtenir ensuite : démarrer le monde sur le Z Flip 5, enregistrer modèle/OS/Chrome, poids transféré, mémoire au repos et au zoom, images par seconde, temps d'entrée dans un point, température et résultat après 10 minutes.

## Documents à mettre à jour si Yocthan accepte

- une ADR séparée pour le choix « GitHub, pas de pont IA permanent » ;
- le journal avec les mesures réellement obtenues ;
- `AGENTS.md` uniquement par Claude, pour préciser les rôles et la règle de fusion ;
- `.github/` dans une PR séparée, à partir des exemples ci-dessous.

## Fichiers de cette proposition

- [OUTILLAGE.md](OUTILLAGE.md) : outils vérifiés, droits, prérequis et commandes ;
- [MANQUES.md](MANQUES.md) : lacunes prioritaires et plus petite action utile ;
- [COORDINATION.md](COORDINATION.md) : boîte aux lettres GitHub et prévention des conflits ;
- [PERFORMANCE-RAM.md](PERFORMANCE-RAM.md) : règle de sobriété et protocole de mesure ;
- `exemples/` : modèles non actifs pour issues, PR et GitHub Actions.

## Ce que Yocthan doit décider

1. Valider GitHub comme seul canal entre IA, sans pont direct permanent.
2. Autoriser ou non ADB/Platform Tools et le débogage USB du Z Flip 5.
3. Autoriser Chrome DevTools MCP à la demande, avec un profil Chrome de test vide.
4. Installer Gemini Code Assist sur le dépôt privé, ou continuer par fichier transmis manuellement.
5. Autoriser une prochaine PR qui rend actifs les modèles et contrôles proposés.
