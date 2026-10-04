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

# On attend que les tests existent, puis qu'ils soient tous terminés.
for essai in $(seq 1 80); do
  etats=$(gh pr view "$numero" --json statusCheckRollup -q '[.statusCheckRollup[] | (.conclusion // "") ] | join(" ")')
  nombre=$(gh pr view "$numero" --json statusCheckRollup -q '.statusCheckRollup | length')
  en_cours=$(gh pr view "$numero" --json statusCheckRollup -q '[.statusCheckRollup[] | select((.status // "COMPLETED") != "COMPLETED")] | length')
  if [ "$nombre" -gt 0 ] && [ "$en_cours" -eq 0 ]; then
    break
  fi
  sleep 15
done

if [ "${nombre:-0}" -eq 0 ] || [ "${en_cours:-1}" -ne 0 ]; then
  echo "REFUS : les tests ne sont pas terminés (ou il n'y en a aucun). Rien n'est fusionné."
  exit 1
fi
for etat in $etats; do
  if [ "$etat" != "SUCCESS" ] && [ "$etat" != "SKIPPED" ] && [ "$etat" != "NEUTRAL" ]; then
    echo "REFUS : un test n'est pas vert ($etats). Rien n'est fusionné."
    exit 1
  fi
done
echo "Tests : $nombre terminés, tous verts ($etats)."
gh pr merge "$numero" --merge
