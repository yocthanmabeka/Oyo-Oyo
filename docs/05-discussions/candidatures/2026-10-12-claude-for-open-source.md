# Candidature au programme Claude for Open Source (Anthropic), 2026-10-12

- Écrit par Claude (la session du PC) à la demande de Yocthan, qui colle ces textes dans le formulaire. Les chiffres sont ceux du dépôt le 2026-10-12, relevés par `git` et `gh` : 816 commits sur `main` depuis le 2026-09-21, 223 pull requests fusionnées, 37 issues fermées, 141 leçons, 107 fiches de décision, public depuis le 2026-10-08, 0 étoile et 0 fork. Rien n'est arrondi vers le haut.
- Le formulaire pose ses questions en français ; les réponses sont en anglais, la langue des relecteurs. Une version française peut être donnée.

---

## 1. The project in 150 words

Metaverse_007 is a research and engineering project led by Yocthan Mabeka, a non-programmer, to build HoloCode: a language in which a beginner writes a complete website, and later a 3D world, in one `.holo` file of named blocks, without writing HTML, CSS or JavaScript. A Rust engine compiled to WebAssembly (742 KB in its light build) renders the page, checks every file and refuses a fault with the right word. The `holo serve` server keeps accounts, shared values and messages on the author's own machine: no mandatory provider. Three weeks after going public, the web part is complete: 141 lessons, 121 of 135 web capabilities covered, accessibility audited by axe-core, phone and desktop parity. Claude Code did the engineering: two coordinated sessions (PC and cloud) and sub-agents working from a GitHub issue queue, every decision recorded as an ADR, every merge gated by green CI. Next: 3D, by 2030.

---

## 2. « Parlez-nous de la portée et de l'impact du projet » — Reach and impact

Metaverse_007 is young and honest about it: the repository was created on 2026-09-21 and has been public since 2026-10-08. There are no downloads, no stars and nobody depends on it yet. What exists is a working engine and a complete method, both visible on GitHub.

Activity: 816 commits on `main`, 223 merged pull requests and 37 closed issues in three weeks; 107 architecture decision records (ADR), each with the options compared and the reasons; 141 lessons, one notion each, every one of them re-read by the test suite; a Rust test suite and a 98-test headless-Chrome suite (gestures, screen reader output, phone emulation, axe-core accessibility audit) run on every pull request; a journal of every step, including the mistakes.

Role in the ecosystem: HoloCode is an attempt to give beginners, in any language, the whole of the web without HTML, CSS or JavaScript, with accessibility and phone parity built into the engine rather than left to the author, and with the author's server at home instead of a provider. Payment, notifications and other real-world services are designed as universal sockets into which each country plugs its own means. The repository is also a documented case of AI-led software engineering with full traceability, useful to the Claude Code community in itself.

---

## 3. « Comment utiliserez-vous l'abonnement pour votre projet ? » — How we will use the subscription

Claude Code is the project's engineering team. The maintainer does not write code: he decides, tries every feature on his phone and PC, and validates. Two Claude Code sessions (one on his PC, one in the cloud) coordinate through GitHub issues; sub-agents take tasks from a labelled queue, push their work after every step, and open pull requests that GitHub tests before the PC session merges them.

The subscription will fund the next phase, 3D (`docs/04-roadmap/PLAN-3D.md`): depth and solid objects, realistic light in quality tiers, the first chair, then multiplayer presence on the author's own server; the phone measurements (speed, memory, battery) that gate each step; and the remaining web features opened under conditions (password field, visitor drawing area, push notifications with the author's keys, international payment). The account's session limits currently stop agents mid-task several times a day; a steadier allowance means fewer interruptions and more of the work saved and reviewed instead of restarted.

---

## 4. « Autres informations » — Other information

- The project is led by dictation, in French, by a maintainer who is not a programmer; Claude Code explains, recommends, then builds, and nothing is merged that he has not tried. Code is in English, comments and texts in French (ADR-060).
- Licence MIT since 2026-10-12 (ADR-122); before that the public repository was "all rights reserved". Code of conduct: Contributor Covenant 2.1. Contribution guide in French and English. Two `good first issue` tasks are open.
- The repository was renamed `yocthanmabeka/Metaverse_007` on 2026-10-12; the former names (`Metaverse`, `Oyo-Oyo`) redirect.
- Design rules that shape everything: one way to do each thing; no hidden code, so the engine verifies all of it and strangers can share a world safely; the same description seen flat (a website) and in depth (a world); nothing that works only on the phone or only on the computer; no country and no provider taken as the model, because HoloCode is meant to be international.
- Contact: through GitHub (issues, or the maintainer's profile, @yocthanmabeka).
