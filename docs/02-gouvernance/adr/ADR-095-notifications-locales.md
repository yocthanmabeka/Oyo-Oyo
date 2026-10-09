# ADR-095 — Lot 9 : des notifications locales, `Notification`

- Statut : ACCEPTÉ (validé par Yocthan le 2026-10-09 : intégrer le travail de Codex quand il est bien fait)
- Date : 2026-10-08 (la construction, par Codex) ; 2026-10-09 (la relecture, l'intégration et la validation)
- Responsable : Yocthan Mabeka
- Discussions sources : le plan en neuf lots (`proposals/Claude/tout-le-web-2026-10/SYNTHESE.md`, lot 9 ; les notifications dans les dettes relevées par Codex) ; la reprise du lot 9 par Codex (issue 202) ; la PR 203 et son compte rendu (`proposals/GPT5.6/fin-lot9-2026-10-08/README.md`).
- Validation : Yocthan, le 2026-10-09.
- Projets affectés : HoloCode, HoloEngine
- Auteur de la construction : **Codex** (PR 203) ; relu, intégré et documenté par Claude. Voir aussi `ADR-093`, `ADR-094`, `ADR-096`.

## Contexte

Un rappel (« fais une pause dans dix minutes ») est une notification du système. Sur le web, il faut demander la permission, enregistrer un service worker, écrire `showNotification`, et souvent un service de « push » extérieur ; beaucoup de sites demandent la permission dès l'ouverture, ce que les navigateurs finissent par bloquer.

## Décision

```holo
Page(
  children: [
    Notification(name: Reminder, label: "Local reminder in three seconds", title: "Break", body: "Time to rest.", after: 3s),
    Button(name: Notify, text: "Allow and schedule the reminder"),
    Button(name: Cancel, text: "Cancel the reminder"),
  ],
  rules: [ On(Notify.tap, effect: Reminder.show), On(Cancel.tap, effect: Reminder.stop) ],
)
```

1. **`Notification(name:, label:, title:, body:, after:)`** : un titre de 100 caractères au plus, un texte de 200 au plus, un délai de `0s` à `3600s`.
2. **`Reminder.show`** demande la permission au visiteur (seulement sur le toucher d'un bouton), puis montre la notification, tout de suite ou après le délai. **`Reminder.stop`** annule un rappel prévu.
3. **Locale** : aucun service extérieur, aucun « push ». Le rappel ne vit que **tant que la page reste ouverte** ; il n'est pas promis si la page est fermée, gelée ou le téléphone endormi, et la page le dit.
4. Un refus est annoncé (`Reminder.failed`, « Permission refusée ») et la page reste utilisable. Le service worker du moteur ne s'installe que lorsqu'un visiteur demande une notification ou une copie hors-ligne (`ADR-096`).
5. Comme les autres capacités : directement dans les enfants de `Page`, un nom et une étiquette ; `Reminder.done` ou `Reminder.failed`.

## Comparaison faite avant de choisir

| Question | Options | Choix, et pourquoi |
|---|---|---|
| Après la fermeture | un service de push ; **seulement page ouverte** | un push demande un serveur et un prestataire ; le premier pas reste chez le visiteur, sans rien promettre qu'il ne tient pas |
| Le moment de la permission | à l'ouverture ; **au toucher** | le visiteur sait pourquoi on la lui demande |
| Le mot | `Alert`, `Reminder` ; **`Notification`** | le mot du web et des systèmes, compris de tous ; un rappel est une notification avec un délai |

## Les défauts du web évités

- **La permission demandée à l'ouverture** : ici, au toucher d'un bouton.
- **Une promesse que la page ne tient pas** (un rappel « dans une heure » perdu en silence) : ici, la limite est écrite dans le langage et dans la page.

## Dettes

- Un rappel après la fermeture de la page demanderait un serveur de push : pas avant que Yocthan le décide.
- L'affichage par le système est simulé dans les essais (le service worker est réel) ; à voir sur le téléphone.

## Critères de validation

- Dans Chrome : « lot9 : notification locale autorisée puis refusée (API simulée) » (un rappel annulé ne s'affiche pas ; un rappel autorisé s'affiche une fois, avec son titre ; un refus annoncé).
- Leçon `118-notifications-locales.holo`.
