# HoloCode face aux langages et frameworks du web, de TypeScript à Flutter

Écrit le 2026-10-06 par Claude, à la demande de Yocthan : « comparer le code actuel de HoloCode, avant de commencer la 3D, avec les meilleurs langages et frameworks du web, en commençant par TypeScript et en terminant par Flutter », par pourcentage, avec l'avis des gens et celui de Claude, pour voir « à combien de pour cent on se rapproche de l'excellence », comment faire mieux, et pourquoi certaines notions ne sont pas reprises. La version web doit être « pas parfaite, mais assez utilisable » avant la 3D.

**À lire avant les chiffres.**

- Les notes de Claude sont **des jugements**, pas des mesures, sauf le nombre de lignes. Claude juge son propre travail : il n'est pas neutre. Le même tableau rempli par Codex et Gemini dira plus.
- Les chiffres des enquêtes viennent **d'extraits de recherche** : les pages officielles (Stack Overflow, State of JS) étaient bloquées depuis la session. Ils sont à vérifier avant d'être cités ailleurs.
- HoloCode n'a **aucun avis public** : personne ne l'utilise en dehors du projet. Sa colonne « avis des gens » reste vide jusqu'à l'essai avec cinq débutants.

Les comparaisons précédentes : face à HTML, CSS et JavaScript (`COMPARAISON-WEB.md`, `TABLEAU-WEB.md`) ; face à SolidJS, Rust, Three.js et Unreal (`COMPARATIF-CONCURRENTS.md`).

## 1. Les neuf retenus, de TypeScript à Flutter

Rangés du plus proche de JavaScript au plus proche de HoloCode.

| | Ce que c'est | Pourquoi lui |
|---|---|---|
| **TypeScript** | JavaScript avec des types, vérifiés avant de lancer | « TypeScript a gagné » : 40 % des développeurs JavaScript n'écrivent plus qu'en TypeScript (State of JS 2025) |
| **React** (et Next.js) | Des composants écrits en JSX, du HTML dans le code | Le plus utilisé de loin |
| **Angular** | Le cadre complet de Google, pour les grandes équipes | Le plus « entreprise » |
| **Vue** | Des composants en un fichier : modèle, code, style | Réputé le plus doux à apprendre des quatre grands |
| **Svelte** | Un compilateur : on écrit presque du HTML | Parmi les plus admirés (Stack Overflow 2026 : 59 %, SvelteKit 64 %) |
| **SolidJS** | Comme React à l'œil, mais ne redessine que ce qui change | Très bien placé en satisfaction (State of JS 2025) |
| **Astro** | Des sites de contenu qui n'envoient presque pas de JavaScript | Le méta-framework préféré (State of JS 2025) |
| **Elm** | Un langage pur, sans erreur à l'exécution | L'inventeur de l'« arbitre » que HoloCode a repris |
| **Flutter** (Dart) | Des widgets, un arbre de blocs, une seule base pour téléphone et web | Ce que Yocthan maîtrise ; HoloCode écrit ses noms comme lui (`ADR-037`) |

Écartés, et pourquoi : Qwik (satisfaction en recul, apprentissage raide), Lit / Web Components (satisfaction modeste), htmx (très admiré, mais il ajoute des attributs au HTML plutôt que d'être un langage ; c'est le plus proche d'esprit de HoloCode, on le garde comme repère).

## 2. La même petite appli, dans les dix

Pour comparer sur pièce : **une liste de tâches**. Un champ, un bouton « Ajouter », une ligne par tâche avec un bouton « Fait » qui la retire, le nombre de tâches (ou « Rien à faire. Bravo ! »), un bouton « Tout effacer », et la liste **gardée d'une visite à l'autre**. C'est la leçon 68 (`exemples/lecons/68-liste-qui-change.holo`).

### HoloCode — 22 lignes, sans styles

```holo
Page(
  title: "Une liste qui change",
  state: State(tache: "", taches: ["Encadrer le tableau de la rivière"]),
  keep: [taches],
  children: [
    H1("Une liste qui change"),
    Row(gap: 8px, children: [
      Input(value: tache, label: "Une tâche"),
      Button(name: Ajouter, text: "Ajouter"),
    ]),
    If(taches, is: 0,
      children: [ P("Rien à faire. Bravo !") ],
      else: [ P("{taches} tâche(s) à faire :") ],
    ),
    Repeat(over: taches, children: [
      Row(gap: 8px, children: [ Text("{item}"), Button(name: Fait, text: "Fait") ]),
    ], rules: [ On(Fait.tap, effect: taches.remove(item)) ]),
    Button(name: Vider, text: "Tout effacer"),
  ],
  rules: [
    On(Ajouter.tap, effect: [taches.push(tache), tache.set("")]),
    On(Vider.tap, effect: taches.clear()),
  ],
)
```

### TypeScript, sans framework — environ 33 lignes (24 de code, 9 de HTML)

```ts
const CLE = "taches";
let taches: string[] = JSON.parse(localStorage.getItem(CLE) ?? '["Encadrer le tableau de la rivière"]');
const champ = document.querySelector<HTMLInputElement>("#tache")!;
const liste = document.querySelector<HTMLUListElement>("#liste")!;
const resume = document.querySelector<HTMLParagraphElement>("#resume")!;

function afficher(): void {
  localStorage.setItem(CLE, JSON.stringify(taches));
  resume.textContent = taches.length === 0 ? "Rien à faire. Bravo !" : `${taches.length} tâche(s) à faire :`;
  liste.replaceChildren(...taches.map((t, i) => {
    const li = document.createElement("li");
    const bouton = document.createElement("button");
    bouton.textContent = "Fait";
    bouton.onclick = () => { taches.splice(i, 1); afficher(); };
    li.append(t, " ", bouton);
    return li;
  }));
}
document.querySelector("#ajouter")!.addEventListener("click", () => {
  if (champ.value.trim()) taches.push(champ.value);
  champ.value = "";
  afficher();
});
document.querySelector("#vider")!.addEventListener("click", () => { taches = []; afficher(); });
afficher();
```

Plus la page HTML (le champ, les deux boutons, le paragraphe, la liste vide).

### React — environ 22 lignes

```tsx
import { useEffect, useState } from "react";

export default function Taches() {
  const [tache, setTache] = useState("");
  const [taches, setTaches] = useState<string[]>(() =>
    JSON.parse(localStorage.getItem("taches") ?? '["Encadrer le tableau de la rivière"]'));
  useEffect(() => localStorage.setItem("taches", JSON.stringify(taches)), [taches]);

  return (
    <main>
      <h1>Une liste qui change</h1>
      <label>Une tâche <input value={tache} onChange={e => setTache(e.target.value)} /></label>
      <button onClick={() => { if (tache) setTaches([...taches, tache]); setTache(""); }}>Ajouter</button>
      <p>{taches.length === 0 ? "Rien à faire. Bravo !" : `${taches.length} tâche(s) à faire :`}</p>
      <ul>
        {taches.map((t, i) => (
          <li key={i}>{t} <button onClick={() => setTaches(taches.filter((_, j) => j !== i))}>Fait</button></li>
        ))}
      </ul>
      <button onClick={() => setTaches([])}>Tout effacer</button>
    </main>
  );
}
```

Piège connu : avec Next.js, lire `localStorage` au premier rendu casse le rendu sur le serveur ; il faut un détour de plus.

### Angular — environ 27 lignes

```ts
import { Component, signal, effect } from "@angular/core";
import { FormsModule } from "@angular/forms";

@Component({
  selector: "app-taches",
  standalone: true,
  imports: [FormsModule],
  template: `
    <h1>Une liste qui change</h1>
    <label>Une tâche <input [(ngModel)]="tache" /></label>
    <button (click)="ajouter()">Ajouter</button>
    @if (taches().length === 0) { <p>Rien à faire. Bravo !</p> }
    @else { <p>{{ taches().length }} tâche(s) à faire :</p> }
    <ul>
      @for (t of taches(); track $index) {
        <li>{{ t }} <button (click)="retirer($index)">Fait</button></li>
      }
    </ul>
    <button (click)="taches.set([])">Tout effacer</button>
  `,
})
export class TachesComponent {
  tache = "";
  taches = signal<string[]>(JSON.parse(localStorage.getItem("taches") ?? '["Encadrer le tableau de la rivière"]'));
  constructor() { effect(() => localStorage.setItem("taches", JSON.stringify(this.taches()))); }
  ajouter() { if (this.tache) this.taches.update(l => [...l, this.tache]); this.tache = ""; }
  retirer(i: number) { this.taches.update(l => l.filter((_, j) => j !== i)); }
}
```

### Vue — environ 18 lignes

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
const tache = ref("");
const taches = ref<string[]>(JSON.parse(localStorage.getItem("taches") ?? '["Encadrer le tableau de la rivière"]'));
watch(taches, v => localStorage.setItem("taches", JSON.stringify(v)), { deep: true });
function ajouter() { if (tache.value) taches.value.push(tache.value); tache.value = ""; }
</script>

<template>
  <h1>Une liste qui change</h1>
  <label>Une tâche <input v-model="tache" /></label>
  <button @click="ajouter">Ajouter</button>
  <p>{{ taches.length === 0 ? "Rien à faire. Bravo !" : `${taches.length} tâche(s) à faire :` }}</p>
  <ul>
    <li v-for="(t, i) in taches" :key="i">{{ t }} <button @click="taches.splice(i, 1)">Fait</button></li>
  </ul>
  <button @click="taches = []">Tout effacer</button>
</template>
```

### Svelte 5 — environ 15 lignes

```svelte
<script lang="ts">
  let tache = $state("");
  let taches = $state<string[]>(JSON.parse(localStorage.getItem("taches") ?? '["Encadrer le tableau de la rivière"]'));
  $effect(() => localStorage.setItem("taches", JSON.stringify(taches)));
</script>

<h1>Une liste qui change</h1>
<label>Une tâche <input bind:value={tache} /></label>
<button onclick={() => { if (tache) taches.push(tache); tache = ""; }}>Ajouter</button>
<p>{taches.length === 0 ? "Rien à faire. Bravo !" : `${taches.length} tâche(s) à faire :`}</p>
<ul>
  {#each taches as t, i}
    <li>{t} <button onclick={() => taches.splice(i, 1)}>Fait</button></li>
  {/each}
</ul>
<button onclick={() => (taches = [])}>Tout effacer</button>
```

### SolidJS — environ 21 lignes

```tsx
import { createSignal, createEffect, For, Show } from "solid-js";

export default function Taches() {
  const [tache, setTache] = createSignal("");
  const [taches, setTaches] = createSignal<string[]>(
    JSON.parse(localStorage.getItem("taches") ?? '["Encadrer le tableau de la rivière"]'));
  createEffect(() => localStorage.setItem("taches", JSON.stringify(taches())));
  return (
    <main>
      <h1>Une liste qui change</h1>
      <label>Une tâche <input value={tache()} onInput={e => setTache(e.currentTarget.value)} /></label>
      <button onClick={() => { if (tache()) setTaches([...taches(), tache()]); setTache(""); }}>Ajouter</button>
      <Show when={taches().length > 0} fallback={<p>Rien à faire. Bravo !</p>}>
        <p>{taches().length} tâche(s) à faire :</p>
      </Show>
      <ul><For each={taches()}>{(t, i) =>
        <li>{t} <button onClick={() => setTaches(taches().filter((_, j) => j !== i()))}>Fait</button></li>
      }</For></ul>
      <button onClick={() => setTaches([])}>Tout effacer</button>
    </main>
  );
}
```

### Astro

Astro fabrique des pages fixes. Pour une partie qui bouge, il pose un « îlot » écrit avec un autre framework (React, Svelte, Vue…) : la liste de tâches s'écrirait donc comme ci-dessus, dans un îlot. HoloCode fait la même chose à sa façon : la page légère arrive seule, et le moteur ne se charge qu'au premier geste qui en a besoin (`ADR-033`).

### Elm — environ 50 lignes, plus un petit fichier JavaScript

Elm ne touche pas au stockage du navigateur lui-même : il faut un « port » vers JavaScript pour garder la liste. Le cœur, en abrégé :

```elm
type alias Model = { tache : String, taches : List String }
type Msg = Ecrire String | Ajouter | Fait Int | Vider

update : Msg -> Model -> ( Model, Cmd Msg )
update msg m =
  case msg of
    Ecrire t -> ( { m | tache = t }, Cmd.none )
    Ajouter  -> let l = if m.tache == "" then m.taches else m.taches ++ [ m.tache ]
                in ( { m | tache = "", taches = l }, sauver l )
    Fait i   -> let l = List.take i m.taches ++ List.drop (i + 1) m.taches in ( { m | taches = l }, sauver l )
    Vider    -> ( { m | taches = [] }, sauver [] )
-- plus : view (≈ 20 lignes), init, main, le port `sauver`, et le JavaScript qui l'écoute.
```

### Flutter (Dart) — environ 38 lignes, plus les imports

```dart
class Taches extends StatefulWidget {
  const Taches({super.key});
  @override
  State<Taches> createState() => _TachesState();
}

class _TachesState extends State<Taches> {
  final champ = TextEditingController();
  List<String> taches = ["Encadrer le tableau de la rivière"];

  @override
  void initState() {
    super.initState();
    SharedPreferences.getInstance().then((p) =>
        setState(() => taches = p.getStringList("taches") ?? taches));
  }

  void changer(List<String> nouvelles) {
    setState(() => taches = nouvelles);
    SharedPreferences.getInstance().then((p) => p.setStringList("taches", taches));
  }

  @override
  Widget build(BuildContext context) => Column(children: [
        Text("Une liste qui change", style: Theme.of(context).textTheme.headlineMedium),
        Row(children: [
          Expanded(child: TextField(controller: champ, decoration: const InputDecoration(labelText: "Une tâche"))),
          ElevatedButton(onPressed: () {
            if (champ.text.isNotEmpty) changer([...taches, champ.text]);
            champ.clear();
          }, child: const Text("Ajouter")),
        ]),
        Text(taches.isEmpty ? "Rien à faire. Bravo !" : "${taches.length} tâche(s) à faire :"),
        for (final (i, t) in taches.indexed)
          Row(children: [Text(t), TextButton(onPressed: () => changer([...taches]..removeAt(i)), child: const Text("Fait"))]),
        TextButton(onPressed: () => changer([]), child: const Text("Tout effacer")),
      ]);
}
```

Plus le paquet `shared_preferences` à ajouter au projet.

### Ce que montre la liste de tâches

| | Lignes (env.) | Code à écrire | À installer | Garder la liste |
|---|---|---|---|---|
| Svelte | 15 | oui | Node, npm, un projet | 1 ligne d'effet |
| Vue | 18 | oui | Node, npm, un projet | 1 ligne d'effet |
| SolidJS | 21 | oui | Node, npm, un projet | 1 ligne d'effet |
| **HoloCode** | **22** | **non** | **rien** | **`keep: [taches]`** |
| React | 22 | oui | Node, npm, un projet | 1 ligne d'effet (piège avec Next.js) |
| Angular | 27 | oui | Node, npm, l'outil Angular | 1 ligne d'effet |
| TypeScript | 33 | oui | Node, npm, un compilateur | à la main |
| Flutter | 38 | oui | le SDK Flutter, un paquet | à la main, en asynchrone |
| Elm | 50 | oui | Elm, et un port JavaScript | à la main, par un port |

HoloCode n'est pas le plus court : Svelte et Vue font moins de lignes. Mais c'est **le seul où l'on n'écrit aucune ligne de code** (pas de flèche `=>`, pas de `if`, pas d'accolades, pas de `[...taches, tache]`), et le seul qui ne demande **rien à installer**.

## 3. L'avis des gens, en pourcentage

**Corrigé le 2026-10-06 avec les pages officielles lues par Codex** (PR 124, § 14) : la colonne « Admiré (SO 2026) » est vérifiée ; les autres viennent d'extraits de recherche et restent à vérifier. Une erreur du premier jet : en 2025, le framework web le plus admiré de Stack Overflow était Phoenix (environ 79 %), pas Svelte.

Chiffres d'extraits de recherche, non vérifiés sur les pages officielles, sauf la première colonne. « Admiré » : parmi ceux qui l'utilisent, la part qui veut continuer (Stack Overflow 2025, environ 49 000 réponses). « Satisfaction » : la même idée chez State of JS 2025 (environ 12 000 réponses, publiée début 2026). Stack Overflow 2026 est sortie le jour même (2026-10-06) ; ses chiffres par framework n'étaient pas encore repris.

| | **Admiré (SO 2026, vérifié)** | Admiré (SO 2025, extrait) | Satisfaction (State of JS 2025, extrait) | Utilisé (SO 2026, vérifié) |
|---|---|---|---|---|
| TypeScript | — | ~58 % | — (77 % du code JavaScript est en TypeScript) | — |
| React | 46,7 % | 52 % | 79 % | 41,5 % |
| Next.js | — | 46 % | 55 % (68 % en 2024 : la plus forte baisse) | — |
| Angular | 42,1 % | 45 % | 65 % | 16,0 % |
| Vue | 47,2 % | 51 % | 75 % | 17,2 % |
| Svelte | **59,4 %** (SvelteKit 64,4 %) | 62 % | 81 à 86 % | 7,8 % |
| SolidJS | 55,5 % | non trouvé | 90 % (non vérifié) | 1,5 % |
| Astro | 58,8 % | non trouvé | 94 % (non vérifié) | 6,2 % |
| Elm | non trouvé | non trouvé | non trouvé | — |
| Flutter / Dart | non trouvé | — | — | — |
| htmx (repère) | 55,9 % | ~73 % (édition incertaine) | — | 5,4 % |
| HoloCode | — | — | — | 0 en dehors du projet |

Ce que les gens disent, en bref :

- **TypeScript** : sûr, devenu la norme ; mais la configuration et les types compliqués fatiguent.
- **React** : partout, l'emploi, c'est le choix par défaut des IA ; mais `useEffect` est la première plainte (37 % dans State of React 2025), suivie des listes de dépendances (21 %).
- **Next.js** : très complet ; mais « une complexité devenue absurde », trop de changements, la dépendance à Vercel.
- **Angular** : modernisé (signals) ; mais lourd, la satisfaction la plus basse des quatre grands.
- **Vue** : doux à apprendre ; moins d'emplois que React.
- **Svelte** : simple et très admiré (le premier des grands, mais derrière Phoenix en 2025) ; écosystème plus petit, migration vers Svelte 5.
- **SolidJS** : le plus satisfaisant, le plus rapide ; peu utilisé, peu de bibliothèques.
- **Astro** : très peu de JavaScript, idéal pour les sites de contenu ; moins fait pour les applis très interactives.
- **Elm** : jamais d'erreur à l'exécution ; un écosystème qui stagne.
- **Flutter** : une seule base de code pour tout ; sur le web, des pages lourdes et dessinées dans un canevas (mal lues par les moteurs de recherche et certains lecteurs d'écran).

## 4. L'avis de Claude, critère par critère

Chaque critère est noté sur 100 : 100 veut dire « l'excellence aujourd'hui sur ce point ». Le poids dit combien le critère compte pour **le public de HoloCode** : des gens qui ne programment pas, ou peu.

| Critère | Poids | TypeScript | React | Angular | Vue | Svelte | SolidJS | Astro | Elm | Flutter | **HoloCode** |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Facile pour un débutant | 20 | 40 | 50 | 35 | 70 | 80 | 55 | 75 | 45 | 65 | **95** |
| Court à écrire | 10 | 35 | 70 | 50 | 80 | 90 | 70 | 75 | 40 | 50 | **95** |
| Erreurs attrapées tôt, bien expliquées | 10 | 80 | 75 | 85 | 70 | 75 | 75 | 70 | 100 | 90 | **90** |
| Les valeurs qui changent (l'état) | 10 | 20 | 65 | 75 | 85 | 85 | 95 | 40 | 85 | 70 | **70** |
| Les composants réutilisables | 10 | 30 | 90 | 85 | 85 | 80 | 80 | 80 | 65 | 95 | **45** |
| Accessibilité et vrai HTML par défaut | 10 | 50 | 55 | 65 | 55 | 75 | 55 | 70 | 50 | 40 | **80** |
| Rapidité et poids | 10 | 95 | 60 | 55 | 75 | 90 | 95 | 100 | 85 | 35 | **60** |
| Ce qu'on peut construire | 10 | 100 | 100 | 100 | 95 | 90 | 85 | 70 | 60 | 95 | **45** |
| Les outils (éditeur, débogueur, tests) | 5 | 95 | 95 | 90 | 85 | 80 | 70 | 80 | 60 | 95 | **35** |
| Les bibliothèques, l'entraide | 5 | 100 | 100 | 85 | 85 | 70 | 45 | 70 | 30 | 85 | **5** |
| **Total** | 100 | **59 %** | **71 %** | **67 %** | **77 %** | **82 %** | **72 %** | **73 %** | **62 %** | **70 %** | **70 %** |

**Avec les poids d'un développeur professionnel** (la facilité compte moins ; ce qu'on peut construire, les outils et l'écosystème comptent plus) : Svelte 82 %, Vue 81 %, React 79 %, Flutter 77 %, Angular 76 %, SolidJS 75 %, Astro 73 %, TypeScript 67 %, Elm 64 %, **HoloCode 58 %**.

### Les notes de HoloCode, expliquées

- **Facile, 95** : un seul fichier, aucun code, rien à installer ; une faute est refusée avec le bon mot, corrigée d'un clic. Pas 100 : 290 mots à connaître, c'est beaucoup.
- **Court, 95** : la liste de tâches en 22 lignes, la mémoire en un mot.
- **Erreurs, 90** : tout est vérifié, avec la ligne et la colonne. Elm reste le modèle (100).
- **État, 70** : `State`, des règles, des listes. Mais seulement des nombres entiers et des textes ; les listes ne contiennent que des textes ; aucune valeur calculée d'après une autre, sauf le nombre et le total d'un panier.
- **Composants, 45** : le plus gros manque. Un `Part` n'a **ni paramètres, ni valeurs, ni règles**. On ne peut pas écrire une « carte d'article » une fois et la poser dix fois avec un titre et un prix différents (seul `Repeat` le fait, dans une même page).
- **Accessibilité, 80** : le texte de remplacement des images est obligatoire, les repères (`Header`, `Main`…) existent, le survol marche au clavier, et la page est du vrai HTML. Pas encore essayée avec un lecteur d'écran.
- **Rapidité et poids, 60** : une page qu'on ne fait que lire pèse 8 Ko ; mais dès qu'elle bouge, le moteur pèse environ 560 Ko, contre 10 à 50 Ko pour Svelte ou Solid.
- **Ce qu'on peut construire, 45** : pas de liste d'articles à champs, pas de liste venue du serveur, pas de nombres à virgule, pas d'envoi vers un vrai serveur, pas de comptes, pas d'envoi de fichier, pas de largeur par élément ni d'élément qui prend la place restante, un seul nom de style par bloc.
- **Outils, 35** : l'éditeur du navigateur, l'extension VS Code, `holo check`. Pas de débogueur pour voir les valeurs, pas de tests pour l'auteur, pas de mise en forme automatique.
- **Entraide, 5** : rien n'existe en dehors de ce dépôt.

## 5. À combien de l'excellence ?

| Mesure | HoloCode aujourd'hui |
|---|---|
| Face au meilleur pour son public (Svelte, 82 %) | **70 / 82 = 85 %** |
| Face à l'excellence absolue (100 partout) | **70 %** |
| Pour un développeur professionnel | **58 %** (Svelte : 82 %) |
| Là où il est le meilleur des dix | facile (95), court (95), accessibilité (80) |
| Là où il est le dernier | composants (45), ce qu'on peut construire (45), outils (35), entraide (5) |

Ce que cela dit : **HoloCode est déjà au niveau de React et de Flutter pour son public**, grâce à sa facilité, mais pas encore « assez utilisable » pour un vrai site : un site d'artisan avec dix produits demande des composants à paramètres et des listes à champs, qu'il n'a pas.

## 6. Ce que HoloCode a pris à chacun

| À qui | Ce que HoloCode a pris |
|---|---|
| TypeScript | Tout vérifier avant d'afficher, avec la ligne de la faute |
| React | Une page faite de blocs emboîtés ; une règle d'affichage qui suit les valeurs |
| Angular | Les formulaires qui envoient, et leurs états (`sent`, `failed`) |
| Vue | Le champ lié à une valeur (`Input(value: tache)`, comme `v-model`) ; le style dans le même fichier |
| Svelte | Écrire presque comme du HTML, le moins de mots possible |
| SolidJS | Ne changer que le texte qui dépend d'une valeur, sans tout redessiner |
| Astro | La page légère d'abord, le moteur seulement quand il sert (`ADR-033`) |
| Elm | L'arbitre pur (un état, un signal, un nouvel état) et les messages d'erreur qui disent quoi faire |
| Flutter | L'arbre de blocs (`Row`, `Column`, `Stack`), `children:`, et l'écriture des noms (`ADR-037`) |

## 7. Ce que HoloCode ne prend pas, et pourquoi

| Notion | Chez qui | Pourquoi HoloCode ne la prend pas |
|---|---|---|
| **Le code libre** : fonctions, boucles, `if` dans le code, flèches `=>` | tous | C'est ce qui rend la programmation difficile et dangereuse. HoloCode a des règles et des demandes ; le calcul libre passe par un module enfermé (`ADR-045`). |
| **Les types écrits par l'auteur** (`string[]`, interfaces, génériques) | TypeScript, Angular, Elm, Dart | Le moteur devine le type d'après la valeur de départ (`State(taches: [])`). Écrire les types est de la verbosité pour un débutant. |
| **`null`, `undefined`, `any`** | TypeScript, JavaScript | Une valeur a toujours un départ ; il n'y a pas de « rien » qui fait planter. |
| **Les effets et leurs dépendances** (`useEffect`, `$effect`, `watch`) | React, Svelte, Vue, Solid | La première plainte des développeurs React (37 %). HoloCode dit ce qu'il veut (`keep: [taches]`), le moteur s'occupe du quand. |
| **Le HTML mélangé au code** (JSX) | React, Solid | On ne sait plus où finit la page et où commence le programme. Les blocs de HoloCode sont la page. |
| **Les classes, décorateurs, injection de dépendances** | Angular, Flutter (`StatefulWidget`) | Des notions d'architecture pour grandes équipes, pas pour quelqu'un qui veut une page. |
| **L'étape de construction, npm, `node_modules`** | tous sauf htmx | Rien à installer : un fichier, un navigateur. Et pas des centaines de paquets tiers, chacun une porte d'entrée pour une attaque. |
| **Le DOM virtuel** | React | Le serveur envoie le HTML déjà fabriqué ; ensuite seul ce qui change est touché (comme Solid). |
| **Les promesses, `async` / `await`** | tous | `Data`, `Form` et `Module` donnent des signaux simples (`done`, `sent`, `failed`). |
| **Dessiner la page dans un canevas** | Flutter web | Une page doit rester du vrai HTML : lisible par les moteurs de recherche, les lecteurs d'écran, la traduction automatique (`ADR-011`). |
| **Les ponts vers JavaScript** | tous | Rejetés (`ADR-011`, partie B) : ils réintroduiraient tout ce qu'on vient d'écarter. |
| **Les classes utilitaires** (Tailwind) | autour de React et Vue | Des dizaines de petits noms à apprendre. HoloCode garde des styles écrits comme du CSS, que tout le web connaît. |

## 8. Comment faire mieux, avant la 3D

Pour que le web soit « assez utilisable » : pouvoir faire **le site d'un artisan, avec un catalogue, un panier et un formulaire de contact, sans aide**. Voici ce qui manque, par ordre d'effet.

| # | Ce qu'on ajoute | Critère qui monte | Effet estimé |
|---|---|---|---|
| 1 | **Des morceaux à paramètres** : `Part(name: Carte, …)` posé avec `Use(Carte, title: "…", price: 120)` ; un morceau peut avoir ses valeurs et ses règles | Composants 45 → 80 | +3,5 points |
| 2 | **Des listes à champs**, dans la page et venues du serveur (`Data` qui remplit une liste d'articles) | État 70 → 85, construire 45 → 60 | +3 points |
| 3 | **La disposition qui manque** : largeur d'un élément, élément qui prend la place restante, plusieurs noms de style par bloc, un fichier de styles à part | Construire 60 → 70, facile reste 95 | +1 point |
| 4 | **Alléger le moteur d'une page qui bouge** : l'arbitre seul, sans le dessin des points (cible : moins de 100 Ko) | Rapidité et poids 60 → 80 | +2 points |
| 5 | **Les outils** : voir les valeurs pendant qu'on essaie la page, une mise en forme automatique, des essais écrits (« quand je touche Ajouter, la liste a une ligne de plus ») | Outils 35 → 60 | +1,25 point |
| 6 | **Un essai avec un lecteur d'écran** (TalkBack sur le téléphone, NVDA sur le PC), et la correction de ce qu'il trouve | Accessibilité 80 → 85 | +0,5 point |
| 7 | **L'essai avec cinq débutants**, déjà prévu : la première colonne « avis des gens » de HoloCode | — | la seule mesure qui compte vraiment |

**Avec les points 1 à 6 : environ 81 %**, au niveau de Svelte (82 %) pour le public de HoloCode, et environ 73 % pour un développeur professionnel. Le dernier écart est l'entraide (5 %) : elle ne se programme pas, elle vient avec des utilisateurs.

**Règle à tenir** (Yocthan, 2026-10-04) : pas de mot nouveau sans un exemple réel qui le demande, et toujours chercher la forme la plus courte. Les six points ci-dessus doivent ajouter peu de mots : `Use` et `Part` existent déjà, il s'agit surtout de leur donner des paramètres.

## 9. Ce que Yocthan a à décider

1. Faire passer ces six points **avant** le premier chantier de la 3D (`ADR-049`) ?
2. Dans quel ordre ? Proposition : 1 (morceaux à paramètres), 2 (listes à champs), 4 (moteur allégé), puis 3, 5, 6.
3. Demander à Codex et à Gemini de remplir le même tableau, pour voir l'écart avec celui de Claude ?

## 10. Le soir du 2026-10-06 : après les six points

Yocthan a validé les six points et donné son feu vert ; ils sont construits et fusionnés : les composants (`ADR-050`), les listes à champs (`ADR-051`), la place qui reste et un thème partagé (`ADR-052`), un moteur léger (`ADR-053`), les outils de l'auteur (`ADR-054`), l'accessibilité vérifiée (`ADR-055`). Codex a relu les composants entre-temps (PR 124) ; sa remarque sur les styles importés est corrigée.

| Critère | Poids | HoloCode le matin | **HoloCode le soir** | Pourquoi |
|---|---|---|---|---|
| Facile pour un débutant | 20 | 95 | **93** | Toujours aucun code ; mais quelques mots de plus (`parts`, `params`, `grow`). |
| Court à écrire | 10 | 95 | **95** | |
| Erreurs attrapées tôt, bien expliquées | 10 | 90 | **92** | Le contraste refusé ; deux styles importés en conflit refusés. |
| Les valeurs qui changent | 10 | 70 | **82** | Des listes à champs, remplies par un geste ou par le serveur. |
| Les composants réutilisables | 10 | 45 | **80** | Paramètres, posés comme un bloc, restylés par le CSS ; pas encore d'emplacement pour du contenu (`slot`). |
| Accessibilité et vrai HTML par défaut | 10 | 80 | **90** | 0 défaut axe-core sur 72 pages dans quatre modes ; l'essai humain reste à faire. |
| Rapidité et poids | 10 | 60 | **70** | Une page qui bouge : 149 Ko au lieu de 626 ; l'objectif de 100 Ko n'est pas atteint. |
| Ce qu'on peut construire | 10 | 45 | **60** | Un catalogue venu du serveur, un panier par composants ; toujours pas de comptes ni d'envoi vers un vrai serveur. |
| Les outils | 5 | 35 | **55** | `?valeurs`, `holo fmt`, `holo essai` ; pas de débogueur. |
| Les bibliothèques, l'entraide | 5 | 5 | **5** | Rien n'a changé : cela vient avec des utilisateurs. |
| **Total** | 100 | **70 %** | **78,5 %** | Svelte : 82 %. |

Pour un développeur professionnel : **58 % → 70 %**.

**À combien de l'excellence ?** 78,5 / 82 = **96 % du meilleur** pour son public, au lieu de 85 % le matin. J'avais annoncé environ 81 % avec ces six points ; le résultat est plus bas, surtout parce que le moteur léger pèse encore 149 Ko et que l'essai humain au lecteur d'écran n'est pas fait. Ces notes restent le jugement de Claude ; Codex, avant les listes à champs, donnait déjà 74,6 % à HoloCode.

**Ce qui reste avant la 3D, par ordre d'effet** : l'essai humain au lecteur d'écran (Yocthan, sur le Flip) ; l'essai avec cinq débutants (la seule mesure qui compte vraiment) ; l'envoi vers un vrai serveur et les comptes ; un emplacement pour du contenu dans un composant ; un moteur d'exécution plus léger encore.

**Depuis** : des valeurs par défaut et des signaux branchés par la page (`ADR-056`), un champ dans une ligne et des lignes gardées (`ADR-057`), un emplacement pour du contenu, `children` (`ADR-058`). Les composants passent de 80 à **88**, les valeurs qui changent de 82 à **85** : le total monte à environ **79,6 %**. L'envoi d'un fichier, puis le serveur et les comptes, restent devant.

## 11. Les trois avis, côte à côte

| | Claude (matin) | Gemini (matin) | Codex (avec les composants) | Claude (soir, après les six points) |
|---|---|---|---|---|
| Svelte | 82 | 83,0 | 85,4 | 82 |
| Vue | 77 | 77,5 | 80,0 | 77 |
| Astro | 73 | 80,5 | 76,8 | 73 |
| SolidJS | 72 | 72,0 | 74,5 | 72 |
| React | 71 | 65,5 | 70,3 | 71 |
| Flutter | 70 | 63,0 | 71,7 | 70 |
| Angular | 67 | 59,0 | 68,3 | 67 |
| Elm | 62 | 62,8 | 65,8 | 62 |
| TypeScript | 59 | 58,0 | 58,8 | 59 |
| **HoloCode** | **70** | **68,5** | **74,6** | **78,5** |

Les trois s'accordent : Svelte en tête ; HoloCode premier sur la facilité et la concision, dernier sur l'entraide. Les réponses : Gemini dans `docs/05-discussions/reponses/2026-10-06-gemini-composants-et-comparatif.md`, Codex dans `proposals/GPT5.6/web-assez-utilisable-2026-10-07/`.

