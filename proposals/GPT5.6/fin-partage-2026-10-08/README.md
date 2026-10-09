# Finir les premières valeurs partagées — 2026-10-08

Sujet : dettes #182, après les comptes (#177, #199, #205, #206).
Décisions concernées : ADR-079 ; numéro réservé ADR-080 ; ADR-060.
Statut proposé : EXPÉRIMENTATION ; aucun statut changé.

## Résultat proposé

Une liste partagée contient jusqu’à cinquante textes de deux cents caractères. Les ajouts partent toujours de la liste du serveur, même si le visiteur envoie une autre copie. `push` et `clear` sont permis. Retirer par numéro de ligne est refusé : une autre personne peut ajouter ou retirer avant la requête ; il faut d’abord des identifiants stables dans ce protocole. Les fiches à champs attendent aussi cette suite. Toutes les valeurs partagées codées restent sous 16 Kio. Dépasser une limite refuse le geste entier, sans modifier ses autres valeurs.

Un `Input(value: title)` peut préparer un texte partagé si un bouton possède `On(Save.tap, effect: title.set(title))`. Dans Chrome, la frappe reste un brouillon séparé ; les textes et les conditions lisent toujours la valeur publiée. Le toucher confirme. Le serveur vérifie d’abord le bouton contre son état, puis remplace seulement l’argument de cette demande explicite par le texte saisi et borné. Le brouillon ne peut donc pas révéler un bouton caché. Même confirmation avec le formulaire sans JavaScript.

Une réponse tardive ne remplace plus une valeur que le visiteur a changée depuis son envoi. Le champ partagé conserve aussi son brouillon quand une publication SSE arrive d’une autre page. Un échec laisse le brouillon disponible.

Le serveur limite les touchers : soixante par visiteur et adresse pendant une minute, cent quatre-vingts par véritable IP. Les compteurs sont bornés et nettoyés ; effacer le cookie ne contourne donc pas le second frein. HTTP 429, attente de soixante secondes, aucune écriture du geste refusé. Un réseau partagé peut atteindre le frein IP ; il ne s’agit pas d’un outil contre une attaque distribuée.

## Preuves

Résultats CI en attente : aucun test annoncé vert dans ce compte rendu. Tests Rust : copie forgée, cinquante ajouts, refus atomique du 51e, texte confirmé, bouton caché, frein par visiteur et IP. Chrome : deux profils distincts et ajouts simultanés ; réponse tenue pendant une nouvelle frappe ; brouillon pendant une publication étrangère ; confirmation sans JS ; 61 requêtes réelles et vérification SQLite.

Le terminal PC reste indisponible. Aucun essai Samsung ni mesure de batterie n’est exécuté ici.

## Écriture

```holo
Page(
 state: State(note: ""),
 shared: Shared(names: [], title: "Titre commun"),
 children: [
  Input(value: note, label: "Un message"),
  Button(name: Add, text: "Ajouter"),
  Repeat(over: names, children: [P("{item}")]),
  Input(value: title, label: "Préparer un titre"),
  Button(name: Save, text: "Publier"),
 ],
 rules: [
  On(Add.tap, effect: [names.push(note), note.set("")]),
  On(Save.tap, effect: title.set(title)),
 ],
)
```

## Limites et remise

Le serveur tranche les valeurs partagées, pas une application arbitraire. L’état personnel d’un membre vient de SQLite ; les champs autorisés sont relus. Les états personnels d’un visiteur anonyme ne constituent toujours pas un contrôle de droit : protéger l’adresse par `access: members` pour ce besoin.

Les leçons 102 et 103 accompagnent les changements ; guide et noms sont complétés. PR en brouillon, empilée après #206, sans fusion, sans modification du journal, d’AGENTS ou des statuts. Claude PC garde la relecture et la fusion.
