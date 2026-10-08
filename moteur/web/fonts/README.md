# Les polices libres du moteur

Une trentaine de polices libres, gardées dans le projet pour qu'une page puisse les nommer sans fichier (`ADR-092`) :

```holo
Page(fonts: [ Font(family: "Inter") ], children: [ … ])
P { font-family: Inter, sans-serif; }
```

- **Chargées seulement quand une page les nomme.** Chaque police a sa feuille, `font.css`, qui dit pour chaque morceau (le latin, le cyrillique, l'arabe…) les caractères qu'il contient. Le navigateur ne télécharge que les morceaux dont la page a besoin : pour une phrase en japonais, un ou deux morceaux sur 124.
- **Toutes les graisses dans un seul fichier** quand la police est variable (« 100 à 900 ») ; l'italique dans un second, quand la police en a un.
- **Licences vérifiées** : toutes sont sous la licence SIL Open Font License 1.1, dont le texte est dans chaque dossier (`LICENSE.txt`), avec le nom de leurs auteurs. Aucune n'a de « nom réservé » : les fichiers découpés pour le web en sont une version modifiée, et la licence interdit alors de garder un nom réservé. Lora, Merriweather, Playfair Display et Dancing Script en avaient un : elles sont remplacées par Literata, Noto Serif, Fraunces et Kalam.
- **D'où elles viennent** : les fichiers de Google Fonts, tels que les distribue [Fontsource](https://fontsource.org) (paquets npm `@fontsource-variable/…` et `@fontsource/…`, version 5.3.0, le 2026-10-08), sans les modifier.
- Ces polices gardent leur licence : elles ne sont pas couvertes par la mention « tous droits réservés » du dépôt (voir `LICENSE` à la racine).

| Police | Genre | Écritures | Graisses | Italique | Taille | Dossier |
|---|---|---|---|---|---|---|
| Atkinson Hyperlegible Next | sans empattements | latin, latin étendu | 200 à 800 | oui | 108 Ko | `atkinson-hyperlegible-next/` |
| Bebas Neue | sans empattements | latin, latin étendu | 400 | non | 22 Ko | `bebas-neue/` |
| Inter | sans empattements | cyrillique, cyrillique étendu, grec, grec étendu, latin, latin étendu, vietnamien | 100 à 900 | oui | 442 Ko | `inter/` |
| Lexend | sans empattements | latin, latin étendu, vietnamien | 100 à 900 | non | 86 Ko | `lexend/` |
| Montserrat | sans empattements | cyrillique, cyrillique étendu, latin, latin étendu, vietnamien | 100 à 900 | oui | 343 Ko | `montserrat/` |
| Noto Sans | sans empattements | cyrillique, cyrillique étendu, devanagari, grec, grec étendu, latin, latin étendu, vietnamien | 100 à 900 | oui | 908 Ko | `noto-sans/` |
| Noto Sans Adlam | sans empattements | adlam, latin, latin étendu | 400 à 700 | non | 56 Ko | `noto-sans-adlam/` |
| Noto Sans Arabic | sans empattements | arabe, latin, latin étendu, maths, symboles | 100 à 900 | non | 246 Ko | `noto-sans-arabic/` |
| Noto Sans Bengali | sans empattements | bengali, latin, latin étendu | 100 à 900 | non | 143 Ko | `noto-sans-bengali/` |
| Noto Sans Devanagari | sans empattements | devanagari, latin, latin étendu | 100 à 900 | non | 157 Ko | `noto-sans-devanagari/` |
| Noto Sans Ethiopic | sans empattements | éthiopien, latin, latin étendu | 100 à 900 | non | 241 Ko | `noto-sans-ethiopic/` |
| Noto Sans Hebrew | sans empattements | cyrillique étendu, grec étendu, hébreu, latin, latin étendu | 100 à 900 | non | 45 Ko | `noto-sans-hebrew/` |
| Noto Sans JP | sans empattements | cyrillique, japonais, latin, latin étendu, vietnamien | 100 à 900 | non | 5,0 Mo | `noto-sans-jp/` |
| Noto Sans KR | sans empattements | cyrillique, coréen, latin, latin étendu, vietnamien | 100 à 900 | non | 3,4 Mo | `noto-sans-kr/` |
| Noto Sans NKo | sans empattements | latin, latin étendu, n'ko | 400 | non | 39 Ko | `noto-sans-nko/` |
| Noto Sans SC | sans empattements | chinois simplifié, cyrillique, latin, latin étendu, vietnamien | 100 à 900 | non | 4,3 Mo | `noto-sans-sc/` |
| Noto Sans Tamil | sans empattements | latin, latin étendu, tamoul | 100 à 900 | non | 97 Ko | `noto-sans-tamil/` |
| Noto Sans Thai | sans empattements | latin, latin étendu, thaï | 100 à 900 | non | 72 Ko | `noto-sans-thai/` |
| Noto Sans Tifinagh | sans empattements | latin, latin étendu, tifinagh | 400 | non | 44 Ko | `noto-sans-tifinagh/` |
| Nunito | sans empattements | cyrillique, cyrillique étendu, latin, latin étendu, vietnamien | 200 à 900 | oui | 281 Ko | `nunito/` |
| Open Sans | sans empattements | cyrillique, cyrillique étendu, grec, grec étendu, hébreu, latin, latin étendu, maths, symboles, vietnamien | 300 à 800 | oui | 603 Ko | `open-sans/` |
| Roboto | sans empattements | cyrillique, cyrillique étendu, grec, grec étendu, latin, latin étendu, maths, symboles, vietnamien | 100 à 900 | oui | 471 Ko | `roboto/` |
| EB Garamond | avec empattements | cyrillique, cyrillique étendu, grec, grec étendu, latin, latin étendu, vietnamien | 400 à 800 | oui | 490 Ko | `eb-garamond/` |
| Fraunces | avec empattements | latin, latin étendu, vietnamien | 100 à 900 | oui | 177 Ko | `fraunces/` |
| Literata | avec empattements | cyrillique, cyrillique étendu, grec, grec étendu, latin, latin étendu, vietnamien | 200 à 900 | oui | 347 Ko | `literata/` |
| Noto Serif | avec empattements | cyrillique, cyrillique étendu, grec, grec étendu, latin, latin étendu, maths, vietnamien | 100 à 900 | oui | 1009 Ko | `noto-serif/` |
| Source Serif 4 | avec empattements | cyrillique, cyrillique étendu, grec, latin, latin étendu, vietnamien | 200 à 900 | oui | 356 Ko | `source-serif-4/` |
| Fira Code | chasse fixe | cyrillique, cyrillique étendu, grec, grec étendu, latin, latin étendu, symboles | 300 à 700 | non | 108 Ko | `fira-code/` |
| JetBrains Mono | chasse fixe | cyrillique, cyrillique étendu, grec, latin, latin étendu, vietnamien | 100 à 800 | oui | 175 Ko | `jetbrains-mono/` |
| Caveat | écrite à la main | cyrillique, cyrillique étendu, latin, latin étendu | 400 à 700 | non | 219 Ko | `caveat/` |
| Kalam | écrite à la main | devanagari, latin, latin étendu | 300, 400, 700 | non | 140 Ko | `kalam/` |
| Pacifico | écrite à la main | cyrillique, cyrillique étendu, latin, latin étendu, vietnamien | 400 | non | 123 Ko | `pacifico/` |

En tout : 32 polices, 20,0 Mo, dont 12,6 Mo pour le chinois, le japonais et le coréen.

## Ajouter une police

1. Vérifier sa licence : SIL Open Font License 1.1, **sans nom réservé** (« Reserved Font Name » dans la première ligne de sa licence).
2. Télécharger son paquet Fontsource et le déballer dans un dossier à part : `npm pack @fontsource-variable/nom`, puis `tar -xzf` dans `paquets/nom/`.
3. `python3 -I moteur/outils/fonts.py paquets moteur/web/fonts` copie ses fichiers, écrit sa feuille et sa licence, et affiche sa fiche.
4. L'ajouter à la liste du moteur (`moteur/src/fonts.rs`) et à ce tableau.
