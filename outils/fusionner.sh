#!/usr/bin/env bash
# Fusionne une pull request seulement quand tous ses tests sont terminés et verts.
#
#     outils/fusionner.sh 46
#
# Pourquoi ce script : le 2026-10-04, une pull request a été fusionnée avant la fin de ses
# tests (la commande d'attente avait rendu « aucun test » trop tôt). Règle du projet : on ne
# fusionne que ce qui marche. Ici, sans tests terminés et verts, pas de fusion.
set -u
numero="${1:?usage : outils/fusionner.sh <numéro de la pull request>}"

gh pr view "$numero" --json number,headRefName,author,title -q '"PR " + (.number|tostring) + " — " + .title + " — branche " + .headRefName + " — auteur " + .author.login' || exit 1

# On attend que les tests existent, puis qu'ils soient tous terminés. L'état est lu en une seule
# fois à chaque tour : le 2026-10-04, trois lectures séparées ont laissé passer un test encore en
# cours (lu « en cours » à la première, « terminé » à la troisième).
nombre=0
etats=""
for essai in $(seq 1 80); do
  lecture=$(gh pr view "$numero" --json statusCheckRollup -q '[.statusCheckRollup[] | if (.status // "COMPLETED") != "COMPLETED" then "EN_COURS" else (.conclusion // .state // "") end | if . == "" then "EN_COURS" else . end] | join(" ")')
  nombre=$(echo $lecture | wc -w)
  etats="$lecture"
  case " $lecture " in *" EN_COURS "*|*" PENDING "*|*" EXPECTED "*) ;; *) [ "$nombre" -gt 0 ] && break ;; esac
  sleep 15
done

if [ "$nombre" -eq 0 ]; then
  echo "REFUS : aucun test n'a été trouvé. Rien n'est fusionné."
  exit 1
fi
for etat in $etats; do
  if [ "$etat" != "SUCCESS" ] && [ "$etat" != "SKIPPED" ] && [ "$etat" != "NEUTRAL" ]; then
    echo "REFUS : un test n'est pas terminé ou pas vert ($etats). Rien n'est fusionné."
    exit 1
  fi
done
echo "Tests : $nombre terminés, tous verts ($etats)."
gh pr merge "$numero" --merge
