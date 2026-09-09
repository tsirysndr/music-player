//! Musical keys: how they are written, and what colour they are drawn.
//!
//! Keys are stored and shown in **traditional notation** — `D`, `Fm`, `A#m` —
//! because that is what every other tool the user has open is showing, and two
//! tools disagreeing on the name of the same key is worse than either notation
//! is better.
//!
//! Camelot is still what the *colour* is built on. The two notations describe
//! the same thing differently: traditional names the tonic, Camelot names the
//! position on the circle of fifths. Position is what matters visually, because
//! neighbours there are the keys that mix — whereas alphabetical neighbours are
//! a semitone apart and sound terrible together. So a key is parsed to a root
//! and a mode, and the colour comes from where that lands on the wheel.

/// A musical key: a tonic, and whether it is major or minor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    /// Pitch class, 0 = C.
    pub root: u8,
    pub is_major: bool,
}

/// Sharps rather than flats, chosen once so the same key is always written the
/// same way. Tags use both and either is accepted on the way in.
const NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Position on the circle of fifths for each chromatic root, counting from C.
/// C major is 8B, so the table is anchored there.
const FIFTHS: [u8; 12] = [8, 3, 10, 5, 12, 7, 2, 9, 4, 11, 6, 1];

impl Key {
    /// Traditional notation: `D`, `Fm`.
    pub fn name(&self) -> String {
        format!(
            "{}{}",
            NAMES[(self.root % 12) as usize],
            if self.is_major { "" } else { "m" }
        )
    }

    /// Camelot notation: `10B`, `4A`.
    ///
    /// Kept because it is what tags are often written in, and because the
    /// number is the wheel position the colour uses.
    pub fn camelot(&self) -> String {
        format!(
            "{}{}",
            self.wheel_position(),
            if self.is_major { "B" } else { "A" }
        )
    }

    /// Where this key sits on the circle of fifths, 1..=12.
    ///
    /// A minor key shares its position with the major a minor third above —
    /// they share a key signature and mix freely, which is the whole reason the
    /// wheel is numbered this way.
    pub fn wheel_position(&self) -> u8 {
        let root = if self.is_major {
            self.root % 12
        } else {
            (self.root + 3) % 12
        };
        FIFTHS[root as usize]
    }

    /// Parse either notation. `None` for anything that is not a key — a wrong
    /// key is worse than none, because it gets acted on.
    pub fn parse(value: &str) -> Option<Key> {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }
        from_camelot(value).or_else(|| from_traditional(value))
    }
}

fn from_camelot(value: &str) -> Option<Key> {
    let (number, letter) = value.split_at(value.len().checked_sub(1)?);
    let is_major = match letter {
        "B" | "b" => true,
        "A" | "a" => false,
        _ => return None,
    };
    let number: u8 = number.parse().ok()?;
    if !(1..=12).contains(&number) {
        return None;
    }

    // Invert the wheel table: find the root whose position is this number.
    let major_root = (0..12u8).find(|root| FIFTHS[*root as usize] == number)?;
    Some(Key {
        // A minor key's tonic is a minor third below its relative major's.
        root: if is_major {
            major_root
        } else {
            (major_root + 9) % 12
        },
        is_major,
    })
}

fn from_traditional(value: &str) -> Option<Key> {
    let mut chars = value.chars();
    let natural = match chars.next()?.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    let rest = chars.as_str();
    // Accidentals, in every spelling tags use.
    let (root, rest) = if let Some(rest) = rest.strip_prefix(['#', '♯']) {
        ((natural + 1) % 12, rest)
    } else if let Some(rest) = rest.strip_prefix(['b', '♭']) {
        // `b` after a letter is always a flat: no notation writes a mode as a
        // bare `b`, so there is nothing to be ambiguous with.
        ((natural + 11) % 12, rest)
    } else {
        (natural, rest)
    };

    // A bare letter means major, as every tool that writes one intends.
    let is_major = match rest.trim().to_ascii_lowercase().as_str() {
        "" | "maj" | "major" => true,
        "m" | "min" | "minor" => false,
        _ => return None,
    };

    Some(Key { root, is_major })
}

/// The colour for a key, as `(r, g, b)`.
///
/// Twelve hues evenly around the circle of fifths, so keys that mix look alike
/// and keys that clash do not. Relative major and minor share a position and so
/// a hue, with the minor darker to keep them apart.
///
/// `None` for anything that is not a key. Callers draw nothing for it, which is
/// the honest rendering of "not known".
pub fn rgb_for(key: &str) -> Option<(u8, u8, u8)> {
    let key = Key::parse(key)?;
    let hue = (key.wheel_position() - 1) as f32 * 30.0;
    let value = if key.is_major { 0.92 } else { 0.72 };
    Some(hsv_to_rgb(hue, 0.78, value))
}

/// The colour as `#rrggbb`, for clients that want a string.
pub fn hex_for(key: &str) -> Option<String> {
    let (r, g, b) = rgb_for(key)?;
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

/// Full-saturation hue to rgb. Written out rather than pulled in: it is eight
/// lines, and a colour crate would be a dependency for one function.
fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let sector = (hue / 60.0).rem_euclid(6.0);
    let fraction = sector - sector.floor();
    let p = value * (1.0 - saturation);
    let q = value * (1.0 - saturation * fraction);
    let t = value * (1.0 - saturation * (1.0 - fraction));

    let (r, g, b) = match sector as u32 {
        0 => (value, t, p),
        1 => (q, value, p),
        2 => (p, value, t),
        3 => (p, q, value),
        4 => (t, p, value),
        _ => (value, p, q),
    };
    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two keys that started this, in the notation the user's other tools
    /// show them in.
    #[test]
    fn keys_are_named_the_way_every_other_tool_names_them() {
        assert_eq!(Key::parse("4A").unwrap().name(), "Fm");
        assert_eq!(Key::parse("10B").unwrap().name(), "D");
        assert_eq!(Key::parse("8A").unwrap().name(), "Am");
        assert_eq!(Key::parse("8B").unwrap().name(), "C");
    }

    /// Both notations name the same key, so parsing either must give the same
    /// thing — this is what lets a tag in either spelling be stored once.
    #[test]
    fn the_two_notations_round_trip() {
        for number in 1..=12u8 {
            for letter in ["A", "B"] {
                let camelot = format!("{number}{letter}");
                let key = Key::parse(&camelot).expect(&camelot);
                assert_eq!(key.camelot(), camelot);
                // …and the traditional name parses back to the same key.
                assert_eq!(Key::parse(&key.name()), Some(key), "{camelot}");
            }
        }
    }

    #[test]
    fn traditional_spellings_are_understood() {
        let f_minor = Key::parse("Fm").unwrap();
        assert_eq!(Key::parse("F minor"), Some(f_minor));
        assert_eq!(Key::parse("Fmin"), Some(f_minor));
        assert_eq!(f_minor.camelot(), "4A");

        // The parallel major is a different key, and a long way round the
        // wheel — which is why getting the mode right matters.
        let f_major = Key::parse("F").unwrap();
        assert_eq!(Key::parse("F major"), Some(f_major));
        assert_eq!(f_major.camelot(), "7B");
        assert_ne!(f_major, f_minor);
    }

    #[test]
    fn accidentals_are_understood_in_both_spellings() {
        assert_eq!(Key::parse("Abm"), Key::parse("G#m"));
        assert_eq!(Key::parse("Abm").unwrap().camelot(), "1A");
        assert_eq!(Key::parse("Bb").unwrap().camelot(), "6B");
        assert_eq!(Key::parse("C#").unwrap().camelot(), "3B");
        // Written back out with sharps, chosen once so a key always looks the
        // same wherever it appears.
        assert_eq!(Key::parse("Bb").unwrap().name(), "A#");
    }

    #[test]
    fn nonsense_is_not_a_key() {
        for input in [
            "", "  ", "Hm", "42", "F lydian", "unknown", "13A", "0A", "8C",
        ] {
            assert_eq!(Key::parse(input), None, "{input:?}");
        }
    }

    /// Relative major and minor share a key signature and mix freely, so they
    /// share a wheel position — the number in Camelot, and the hue.
    #[test]
    fn relative_keys_share_a_position() {
        assert_eq!(
            Key::parse("Am").unwrap().wheel_position(),
            Key::parse("C").unwrap().wheel_position()
        );
        assert_eq!(
            Key::parse("Em").unwrap().wheel_position(),
            Key::parse("G").unwrap().wheel_position()
        );
    }

    /// A fifth up is one step round the wheel. That is what makes adjacent
    /// numbers mix, and what the colouring depends on.
    #[test]
    fn a_fifth_up_is_one_step_round_the_wheel() {
        for root in 0..12u8 {
            let here = Key {
                root,
                is_major: true,
            }
            .wheel_position() as i32;
            let fifth = Key {
                root: (root + 7) % 12,
                is_major: true,
            }
            .wheel_position() as i32;
            assert_eq!((fifth - here).rem_euclid(12), 1, "a fifth above {root}");
        }
    }

    #[test]
    fn every_key_has_a_colour_and_a_non_key_has_none() {
        for number in 1..=12 {
            for letter in ["A", "B"] {
                assert!(rgb_for(&format!("{number}{letter}")).is_some());
            }
        }
        assert!(rgb_for("Fm").is_some());
        assert!(rgb_for("D").is_some());
        assert_eq!(rgb_for("nonsense"), None);
        assert_eq!(hex_for("nonsense"), None);
        assert!(hex_for("Fm").unwrap().starts_with('#'));
    }

    /// The whole point of colouring: keys that mix look alike.
    #[test]
    fn keys_that_mix_look_alike() {
        let distance = |a: &str, b: &str| {
            let (r1, g1, b1) = rgb_for(a).unwrap();
            let (r2, g2, b2) = rgb_for(b).unwrap();
            let d = |x: u8, y: u8| (x as i32 - y as i32).pow(2);
            d(r1, r2) + d(g1, g2) + d(b1, b2)
        };
        // Am is 8A; Em is 9A, its neighbour; D#m is 2A, across the wheel.
        assert!(distance("Am", "Em") < distance("Am", "D#m"));
    }

    /// Relative keys share a hue but must not be the same colour, or the two
    /// would be indistinguishable in a list.
    #[test]
    fn relative_keys_are_told_apart_by_lightness() {
        let minor = rgb_for("Am").unwrap();
        let major = rgb_for("C").unwrap();
        assert_ne!(minor, major);
        let brightness = |(r, g, b): (u8, u8, u8)| r as u32 + g as u32 + b as u32;
        assert!(brightness(minor) < brightness(major));
    }
}
