# Fabrique moteur/src/graphemes.rs : la table des classes de caractères dont le moteur a besoin
# pour compter et couper un texte comme une personne le lit (ADR-103), d'après les règles des
# « grappes de graphèmes étendues » d'Unicode (UAX #29), celles de Intl.Segmenter dans Chrome.
#
#     python moteur/outils/graphemes.py        (depuis la racine du dépôt)
#
# Les catégories viennent du module unicodedata de Python (sa version d'Unicode est écrite en tête
# du fichier fabriqué). Ce que Python ne donne pas (les émojis, les préfixes, les consonnes des
# conjointes indiennes) est écrit ici, d'après les fichiers d'Unicode, et commenté.

import sys
import unicodedata
from pathlib import Path

OTHER, EXTEND, SPACING_MARK, CONTROL, PREPEND, PICTOGRAPHIC, REGIONAL, LINKER, CONSONANT, ZWJ, L, V, T = range(13)

classes = [OTHER] * 0x110000


def put(first, last, value):
    for cp in range(first, last + 1):
        classes[cp] = value


# Extended_Pictographic (emoji-data.txt) : seulement utile après un ZWJ (règle GB11).
for first, last in [
    (0x00A9, 0x00A9), (0x00AE, 0x00AE), (0x203C, 0x203C), (0x2049, 0x2049), (0x2122, 0x2122), (0x2139, 0x2139),
    (0x2194, 0x2199), (0x21A9, 0x21AA), (0x231A, 0x231B), (0x2328, 0x2328), (0x2388, 0x2388), (0x23CF, 0x23CF),
    (0x23E9, 0x23F3), (0x23F8, 0x23FA), (0x24C2, 0x24C2), (0x25AA, 0x25AB), (0x25B6, 0x25B6), (0x25C0, 0x25C0),
    (0x25FB, 0x25FE), (0x2600, 0x2605), (0x2607, 0x2612), (0x2614, 0x2685), (0x2690, 0x2705), (0x2708, 0x2712),
    (0x2714, 0x2714), (0x2716, 0x2716), (0x271D, 0x271D), (0x2721, 0x2721), (0x2728, 0x2728), (0x2733, 0x2734),
    (0x2744, 0x2744), (0x2747, 0x2747), (0x274C, 0x274C), (0x274E, 0x274E), (0x2753, 0x2755), (0x2757, 0x2757),
    (0x2763, 0x2767), (0x2795, 0x2797), (0x27A1, 0x27A1), (0x27B0, 0x27B0), (0x27BF, 0x27BF), (0x2934, 0x2935),
    (0x2B05, 0x2B07), (0x2B1B, 0x2B1C), (0x2B50, 0x2B50), (0x2B55, 0x2B55), (0x3030, 0x3030), (0x303D, 0x303D),
    (0x3297, 0x3297), (0x3299, 0x3299), (0x1F000, 0x1F0FF), (0x1F10D, 0x1F10F), (0x1F12F, 0x1F12F),
    (0x1F16C, 0x1F171), (0x1F17E, 0x1F17F), (0x1F18E, 0x1F18E), (0x1F191, 0x1F19A), (0x1F1AD, 0x1F1E5),
    (0x1F201, 0x1F20F), (0x1F21A, 0x1F21A), (0x1F22F, 0x1F22F), (0x1F232, 0x1F23A), (0x1F23C, 0x1F23F),
    (0x1F249, 0x1F3FA), (0x1F400, 0x1F53D), (0x1F546, 0x1F64F), (0x1F680, 0x1F6FF), (0x1F774, 0x1F77F),
    (0x1F7D5, 0x1F7FF), (0x1F80C, 0x1F80F), (0x1F848, 0x1F84F), (0x1F85A, 0x1F85F), (0x1F888, 0x1F88F),
    (0x1F8AE, 0x1F8FF), (0x1F90C, 0x1F93A), (0x1F93C, 0x1F945), (0x1F947, 0x1FAFF), (0x1FC00, 0x1FFFD),
]:
    put(first, last, PICTOGRAPHIC)

# Les marques qui se placent devant une consonne et restent avec elle (Prepend).
PREPENDED = [(0x0600, 0x0605), (0x06DD, 0x06DD), (0x070F, 0x070F), (0x0890, 0x0891), (0x08E2, 0x08E2), (0x0D4E, 0x0D4E),
             (0x110BD, 0x110BD), (0x110CD, 0x110CD), (0x111C2, 0x111C3), (0x1193F, 0x1193F), (0x11941, 0x11941),
             (0x11A3A, 0x11A3A), (0x11A84, 0x11A89), (0x11D46, 0x11D46), (0x11F02, 0x11F02)]
prepended = {cp for first, last in PREPENDED for cp in range(first, last + 1)}

# Les signes qui ne s'écrivent pas : contrôles, séparateurs de ligne et de paragraphe, et les
# formats (Cf), sauf ZWNJ et ZWJ, les préfixes et les étiquettes des drapeaux.
for cp in range(0x110000):
    category = unicodedata.category(chr(cp))
    if category in ("Cc", "Zl", "Zp") or (category == "Cf" and cp not in (0x200C, 0x200D) and cp not in prepended and not 0xE0020 <= cp <= 0xE007F):
        classes[cp] = CONTROL
for first, last in PREPENDED:
    put(first, last, PREPEND)

# Les voyelles et signes qui prennent de la place (Spacing_Mark), sauf ceux qu'UAX #29 laisse à part.
NOT_SPACING = [(0x102B, 0x102C), (0x1038, 0x1038), (0x1062, 0x1064), (0x1067, 0x106D), (0x1083, 0x1083), (0x1087, 0x108C),
               (0x108F, 0x108F), (0x109A, 0x109C), (0x1A61, 0x1A61), (0x1A63, 0x1A64), (0xAA7B, 0xAA7B), (0xAA7D, 0xAA7D),
               (0x11720, 0x11721)]
not_spacing = {cp for first, last in NOT_SPACING for cp in range(first, last + 1)}
for cp in range(0x110000):
    if unicodedata.category(chr(cp)) == "Mc" and cp not in not_spacing:
        classes[cp] = SPACING_MARK
put(0x0E33, 0x0E33, SPACING_MARK)
put(0x0EB3, 0x0EB3, SPACING_MARK)

# Ce qui s'accroche au caractère d'avant (Extend) : les marques (Mn, Me), ZWNJ, les modificateurs
# de couleur de peau, les étiquettes des drapeaux, les marques sonores du katakana, et les
# voyelles indiennes qu'Unicode range dans Other_Grapheme_Extend.
for cp in range(0x110000):
    if unicodedata.category(chr(cp)) in ("Mn", "Me"):
        classes[cp] = EXTEND
for first, last in [(0x200C, 0x200C), (0x1F3FB, 0x1F3FF), (0xE0020, 0xE007F), (0xFF9E, 0xFF9F), (0x09BE, 0x09BE),
                    (0x09D7, 0x09D7), (0x0B3E, 0x0B3E), (0x0B57, 0x0B57), (0x0BBE, 0x0BBE), (0x0BD7, 0x0BD7),
                    (0x0CC2, 0x0CC2), (0x0CD5, 0x0CD6), (0x0D3E, 0x0D3E), (0x0D57, 0x0D57), (0x0DCF, 0x0DCF),
                    (0x0DDF, 0x0DDF), (0x1B35, 0x1B35), (0x302E, 0x302F), (0x1133E, 0x1133E), (0x11357, 0x11357),
                    (0x114B0, 0x114B0), (0x114BD, 0x114BD), (0x115AF, 0x115AF), (0x11930, 0x11930), (0x1D165, 0x1D165),
                    (0x1D16E, 0x1D172)]:
    put(first, last, EXTEND)

put(0x200D, 0x200D, ZWJ)
put(0x1F1E6, 0x1F1FF, REGIONAL)

# Le coréen écrit en jamos (les syllabes toutes faites, AC00 à D7A3, sont calculées par le moteur).
put(0x1100, 0x115F, L)
put(0xA960, 0xA97C, L)
put(0x1160, 0x11A7, V)
put(0xD7B0, 0xD7C6, V)
put(0x11A8, 0x11FF, T)
put(0xD7CB, 0xD7FB, T)

# Les conjointes indiennes (règle GB9c, Unicode 15.1) : une consonne, le virama, une consonne
# restent une seule lettre (« क्ष »). Les viramas et les consonnes de six écritures.
for cp in (0x094D, 0x09CD, 0x0ACD, 0x0B4D, 0x0C4D, 0x0D4D):
    classes[cp] = LINKER
for first, last in [(0x0915, 0x0939), (0x0958, 0x095F), (0x0978, 0x097F),
                    (0x0995, 0x09A8), (0x09AA, 0x09B0), (0x09B2, 0x09B2), (0x09B6, 0x09B9), (0x09DC, 0x09DD), (0x09DF, 0x09DF), (0x09F0, 0x09F1),
                    (0x0A95, 0x0AA8), (0x0AAA, 0x0AB0), (0x0AB2, 0x0AB3), (0x0AB5, 0x0AB9), (0x0AF9, 0x0AF9),
                    (0x0B15, 0x0B28), (0x0B2A, 0x0B30), (0x0B32, 0x0B33), (0x0B35, 0x0B39), (0x0B5C, 0x0B5D), (0x0B5F, 0x0B5F), (0x0B71, 0x0B71),
                    (0x0C15, 0x0C28), (0x0C2A, 0x0C39), (0x0C58, 0x0C5A),
                    (0x0D15, 0x0D3A)]:
    put(first, last, CONSONANT)

# Des plages de même classe, coupées en morceaux de 128 au plus : chaque entrée tient dans un
# nombre de 32 bits, le début (21 bits), la longueur moins un (7 bits) et la classe (4 bits).
entries = []
cp = 0
while cp < 0x110000:
    value = classes[cp]
    if value == OTHER:
        cp += 1
        continue
    first = cp
    while cp < 0x110000 and classes[cp] == value and cp - first < 128:
        cp += 1
    entries.append((first << 11) | ((cp - first - 1) << 4) | value)

lines = []
for i in range(0, len(entries), 8):
    lines.append("    " + " ".join(f"0x{e:08X}," for e in entries[i:i + 8]))

out = Path(__file__).resolve().parent.parent / "src" / "graphemes.rs"
out.write_text(f"""//! La table des classes de caractères pour compter et couper un texte comme une personne le lit
//! (ADR-103), d'après UAX #29. Fabriquée par `moteur/outils/graphemes.py` (Unicode {unicodedata.unidata_version}) :
//! ne pas l'écrire à la main.
//!
//! Chaque entrée : le premier caractère (21 bits), la longueur moins un (7 bits), la classe (4 bits).

/// Les classes : 0 autre, 1 Extend, 2 SpacingMark, 3 Control, 4 Prepend, 5 Extended_Pictographic,
/// 6 Regional_Indicator, 7 virama (InCB Linker), 8 consonne (InCB Consonant), 9 ZWJ, 10 L, 11 V, 12 T.
pub const TABLE: &[u32] = &[
{chr(10).join(lines)}
];
""", encoding="utf-8", newline="\n")
print(f"{len(entries)} plages, Unicode {unicodedata.unidata_version} → {out}", file=sys.stderr)
