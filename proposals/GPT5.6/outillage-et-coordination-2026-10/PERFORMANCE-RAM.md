# Sobriété : performance et RAM

## Règles

1. **Zéro démon permanent** : MCP en transport local `stdio`, lancé par le client seulement quand demandé, puis arrêté.
2. **Un navigateur automatisé à la fois** : Chrome DevTools pour diagnostiquer ; Playwright pour rejouer un test. Jamais les deux simultanément sans raison mesurée.
3. **Pas d'Android Studio** pour la première preuve : Platform Tools/ADB suffit.
4. **Pas de modèle d'IA local** pour coordonner les IA : GitHub transporte du texte et ne consomme aucune RAM permanente sur le PC.
5. **Pas de Blender/Figma/audio pendant le sprint Big Bang** : contenu procédural et outils ciblés.
6. **Mesurer avant/après** : aucun outil n'est déclaré « léger » sur la seule réputation.

## Budget de travail proposé

Les limites ci-dessous sont des critères à valider, pas des mesures déjà obtenues :

- téléphone : pic total de l'onglet inférieur au plafond de projet, avec alerte dès 400 Mo et objectif de travail nettement inférieur ;
- transfert initial : ne pas régresser au-delà de la référence annoncée de 502 Ko sans justification ;
- CI documentaire : aucun navigateur ;
- test visuel : un navigateur, un worker, deux tailles d'écran au maximum ;
- captures : conserver uniquement échec + référence approuvée, avec expiration des artefacts temporaires ;
- chaque nouvel MCP doit remplacer au moins une manipulation pénible et avoir un responsable, un droit minimal et une procédure de désactivation.

## Protocole Samsung Z Flip 5

1. Noter modèle, version Android, version Chrome, commit et URL testée.
2. Fermer les autres onglets/apps, redémarrer Chrome, attendre 30 secondes.
3. Relier en USB, activer le débogage seulement pour le PC autorisé, ouvrir `chrome://inspect#devices`.
4. Enregistrer : octets réseau, temps de chargement, mémoire au repos, mémoire et fluidité pendant 20 zooms, repli WebGPU/WebGL, erreurs console.
5. Répéter trois fois ; publier les trois valeurs, pas seulement la meilleure.
6. Révoquer l'autorisation USB après la campagne si elle n'est plus utile.

Pour une connexion protocolaire directe, Google documente `adb devices -l` puis `adb forward tcp:9222 localabstract:chrome_devtools_remote`. Toute autre commande de mesure mémoire doit d'abord être testée sur ce modèle et documentée ; le nom de processus Chrome varie, donc aucune commande non vérifiée n'est présentée ici comme universelle.
