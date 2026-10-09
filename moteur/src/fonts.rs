//! Les polices libres du moteur (ADR-092) : une trentaine, gardées dans le projet
//! (`moteur/web/fonts/`), pour les écritures du monde entier, chacune avec sa licence (SIL Open
//! Font License 1.1) dans son dossier. Une page en nomme une sans fichier, `Font(family:
//! "Inter")` : le moteur la charge, et le navigateur ne télécharge que les morceaux dont la page
//! a besoin (le latin, le cyrillique, l'arabe…). Une page qui n'en nomme pas ne charge rien.

/// Une police du moteur : son nom, et son dossier dans `moteur/web/fonts/`.
pub struct Font {
    pub family: &'static str,
    pub folder: &'static str,
}

/// Les polices du moteur. Leur liste détaillée (écritures, graisses, licences, tailles) est dans
/// `moteur/web/fonts/README.md`.
pub const LIBRARY: &[Font] = &[
    // Sans empattements.
    Font { family: "Atkinson Hyperlegible Next", folder: "atkinson-hyperlegible-next" },
    Font { family: "Bebas Neue", folder: "bebas-neue" },
    Font { family: "Inter", folder: "inter" },
    Font { family: "Lexend", folder: "lexend" },
    Font { family: "Montserrat", folder: "montserrat" },
    Font { family: "Noto Sans", folder: "noto-sans" },
    Font { family: "Noto Sans Adlam", folder: "noto-sans-adlam" },
    Font { family: "Noto Sans Arabic", folder: "noto-sans-arabic" },
    Font { family: "Noto Sans Bengali", folder: "noto-sans-bengali" },
    Font { family: "Noto Sans Devanagari", folder: "noto-sans-devanagari" },
    Font { family: "Noto Sans Ethiopic", folder: "noto-sans-ethiopic" },
    Font { family: "Noto Sans Hebrew", folder: "noto-sans-hebrew" },
    Font { family: "Noto Sans JP", folder: "noto-sans-jp" },
    Font { family: "Noto Sans KR", folder: "noto-sans-kr" },
    Font { family: "Noto Sans NKo", folder: "noto-sans-nko" },
    Font { family: "Noto Sans SC", folder: "noto-sans-sc" },
    Font { family: "Noto Sans Tamil", folder: "noto-sans-tamil" },
    Font { family: "Noto Sans Thai", folder: "noto-sans-thai" },
    Font { family: "Noto Sans Tifinagh", folder: "noto-sans-tifinagh" },
    Font { family: "Nunito", folder: "nunito" },
    Font { family: "Open Sans", folder: "open-sans" },
    Font { family: "Roboto", folder: "roboto" },
    // Avec empattements.
    Font { family: "EB Garamond", folder: "eb-garamond" },
    Font { family: "Fraunces", folder: "fraunces" },
    Font { family: "Literata", folder: "literata" },
    Font { family: "Noto Serif", folder: "noto-serif" },
    Font { family: "Source Serif 4", folder: "source-serif-4" },
    // À chasse fixe, pour le code.
    Font { family: "Fira Code", folder: "fira-code" },
    Font { family: "JetBrains Mono", folder: "jetbrains-mono" },
    // Écrites à la main.
    Font { family: "Caveat", folder: "caveat" },
    Font { family: "Kalam", folder: "kalam" },
    Font { family: "Pacifico", folder: "pacifico" },
];

/// La police du moteur de ce nom, sans tenir compte des majuscules : « inter » → Inter.
pub fn find(family: &str) -> Option<&'static Font> {
    LIBRARY.iter().find(|font| font.family.eq_ignore_ascii_case(family))
}

/// Les noms des polices du moteur, pour un message ou l'éditeur.
pub fn families() -> Vec<&'static str> {
    LIBRARY.iter().map(|font| font.family).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_font_of_the_library_is_kept_with_its_licence() {
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web").join("fonts");
        for font in super::LIBRARY {
            let here = folder.join(font.folder);
            let licence = std::fs::read_to_string(here.join("LICENSE.txt")).unwrap_or_default();
            assert!(licence.contains("SIL Open Font License, Version 1.1"), "{} : licence absente", font.family);
            let css = std::fs::read_to_string(here.join("font.css")).unwrap_or_default();
            assert!(css.contains(&format!("font-family:\"{}\"", font.family)), "{} : font.css absent", font.family);
            // Chaque fichier nommé par font.css est là.
            for file in css.split("url(\"").skip(1).filter_map(|rest| rest.split('"').next()) {
                assert!(here.join(file).is_file(), "{} : {file} manque", font.family);
            }
        }
        assert!(super::LIBRARY.len() >= 30);
        assert_eq!(super::find("inter").map(|font| font.folder), Some("inter"));
    }
}
