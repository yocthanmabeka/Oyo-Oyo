# Le site de recette du web

Depuis `moteur/` :

```powershell
cargo build --release --bin holo
.\target\release\holo serve ..\exemples\parcours 8080
```

Ouvre http://localhost:8080/ : dix parcours. Aucun service externe requis. Le navigateur reçoit 200 produits, et dessine 20 cartes par page. Les comptes et la base restent sur le PC de l'auteur.

Pour les essais automatisés, après construction des paquets WebAssembly : `node outils/browser-tests.mjs parcours`. Pour la preuve et les limites : [rapport Codex](../../proposals/GPT5.6/web-viable-2026-10-08/README.md).

Sur le vrai téléphone, ouvrir l'adresse du PC indiquée par le serveur, même Wi-Fi. Refaire : catalogue (produit 200, filtre, pages), panier (4 unités : 60,00 €), erreurs/Entrée, photo, menu, deux réservations, connexion sur un second appareil, profil 123, vidéo sous-titrée, lecture des sections et tableau de bord. Avec TalkBack, vérifier l'ordre, les libellés, les erreurs, le retour du focus et la lecture des valeurs. Consigner appareil, Chrome, commit et résultat ; une largeur émulée ne vaut pas ce passage.
