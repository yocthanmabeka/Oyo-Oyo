# Fin du lot 9 : quatre capacités explicites

- Sujet : reprise autorisée par Yocthan le 2026-10-08, « Codex reprend aussi le lot 9 » ; issue #202, passation #189.
- Discussions sources : mandat dans la conversation ; [synthèse des avis](../../Claude/tout-le-web-2026-10/SYNTHESE.md). Aucun numéro HC nouveau inventé.
- Décisions concernées : ADR-016, 035, 037, 060 ; propositions réservées ADR-093 à ADR-096.
- Statut proposé : EXPÉRIMENTATION ; aucun statut d'ADR modifié.

## Résultat proposé

Transfer : JSON local explicite et import atomique. Device : position, presse-papiers, aperçu caméra et microphone local. Notification : permission explicite et rappel limité à la page ouverte. Offline : copie publique bornée, aucune écriture serveur différée.

Le guide et les noms sont complétés. Les leçons 116 à 119 sont dans cette PR. Pas de dépendance ajoutée, pas d'installation sur le PC, pas de nouvelle fenêtre ni changement du serveur local. Base : branche de Claude #195 ; les comptes de #177/#199 et la pagination #201 restent dans leur chaîne séparée, à intégrer et vérifier ensemble ensuite.

## Options comparées

- Un accès libre aux API du navigateur : refusé, il contredirait ADR-015 et le confinement des modules.
- Un bloc par API : plus de noms et de duplication ; le bloc Device expose quatre sortes nommées et contrôlées.
- Une seule capacité universelle : refusée, elle cacherait la portée des permissions. Quatre blocs distinguent un fichier personnel, un appareil, une notification et une copie publique.
- Des rappels après fermeture : pas promis ; un serveur ou un service push serait nécessaire pour une disponibilité durable. Le premier rappel vit dans la page.
- Tout mettre en cache : refusé. Le premier Offline garde une page publique, les ressources déclarées et le moteur léger. Comptes, formulaires et partage ne sont pas éligibles.

## Vérification exécutée

La syntaxe des deux nouveaux scripts JavaScript et du raccord du moteur a été vérifiée dans V8. Ce n'est pas une exécution dans le navigateur. Le terminal du PC échoue au démarrage ; aucune commande Rust ou Chrome exécutée sur le PC dans cette session. Les résultats de GitHub Actions seront ajoutés après leur exécution ; à ce stade ils ne sont pas annoncés verts.

## Objections et limites

La permission et le matériel réels demandent encore un essai sur le Samsung. Les essais de position utilisent DevTools ; les captures utilisent un flux de canvas simulé ; les notifications système seront simulées, avec service worker réel. Ni batterie ni mémoire ni fluidité du téléphone mesurées. Le microphone ne donne pas encore un fichier audio et la caméra ne prend pas une photo. Une demande de permission peut rester pendante si le visiteur ne répond pas ; un changement de page rend sa réponse caduque et arrête les pistes reçues. Un compte ne devient jamais hors-ligne.

Une copie hors-ligne coûte au plus 16 Mio, huit copies au plus (plafond théorique 128 Mio de stockage disque, pas une mesure de mémoire). Le navigateur peut évincer son cache. La copie repart des valeurs initiales au rechargement. Un remplacement qui échoue au stockage retire la copie incomplète ; il ne promet pas la conservation atomique d'une ancienne copie à travers une panne. Aucune synchronisation différée.

## Preuve requise

Rust debug/release, compilation WASM complète et légère, suite Chrome entière, refus des clés et sortes d'import, taille 64 Ko, refus de permission, arrêt des pistes, refus des réponses privées et rechargement réellement sans réseau. TalkBack, clavier complet et téléphone réels restent distincts.

## Sources

[Service workers (MDN)](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API/Using_Service_Workers) ; [permission de notification](https://developer.mozilla.org/en-US/docs/Web/API/Notification/requestPermission_static) ; [getUserMedia](https://developer.mozilla.org/en-US/docs/Web/API/MediaDevices/getUserMedia). Les service workers et l'appareil demandent un contexte sécurisé ; localhost est l'exception locale prévue par les navigateurs. Un serveur externe n'est pas obligatoire.

## Documents à mettre à jour après relecture

Claude : journal, coordination de la reprise, ADR-093 à ADR-096 et registre avec Yocthan. GUIDE et NOMS sont modifiés ici ; aucun changement de TABLEAU-WEB ni de statut de décision.

## Intégration (Claude, session du PC, 2026-10-09)

Intégrée sur `integration/codex-203`, après la PR 201. Écrites : `ADR-093` à `ADR-096`, `ACCEPTÉ` (Yocthan : intégrer le travail de Codex quand il est bien fait). Le guide (section « 6 quatertricies ») et `NOMS.md` les décrivent ; les leçons 116 à 119 suivent la forme des autres et s'enchaînent 115 → 116 → … → 119 → 1. Deux corrections : avec `holo serve` et sans JavaScript, une page `Offline` est servie comme les autres (ses boutons marchent, elle montre les valeurs du visiteur ; le service worker demande la copie sans cookie, et reçoit la page de départ) ; le service worker laisse passer les écritures et le direct (`text/event-stream`). Un test du serveur et un essai dans Chrome les gardent.
