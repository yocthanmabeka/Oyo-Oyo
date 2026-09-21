"""Unités physiques de HoloCode v0.1.

Toute quantité est convertie en unité SI dès la lecture (mètre, seconde) et
garde sa dimension. C'est la dimension, pas le nombre, que le vérificateur
compare : `2m` et `3s` ne sont pas du même type.
"""

LENGTH_UNITS = {"mm": 0.001, "cm": 0.01, "m": 1.0, "km": 1000.0}
DURATION_UNITS = {"ms": 0.001, "s": 1.0, "min": 60.0, "h": 3600.0}

TYPES = ("Bool", "Number", "Text", "Length", "Duration")
ORDERED_TYPES = ("Number", "Length", "Duration")


def dimension(unit):
    """Retourne (type, facteur SI) pour un suffixe d'unité, ou None s'il est inconnu."""
    if unit is None:
        return "Number", 1.0
    if unit in LENGTH_UNITS:
        return "Length", LENGTH_UNITS[unit]
    if unit in DURATION_UNITS:
        return "Duration", DURATION_UNITS[unit]
    return None


def show(type_, value):
    """Écrit une valeur comme elle s'écrirait dans un programme HoloCode."""
    if type_ == "Bool":
        return "true" if value else "false"
    if type_ == "Text":
        return f'"{value}"'
    if type_ == "Length":
        return f"{value:g} m"
    if type_ == "Duration":
        return f"{value:g} s"
    return f"{value:g}"
