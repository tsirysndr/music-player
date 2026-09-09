//! A colour per musical key.
//!
//! The point of colouring a key is that **keys which mix look alike**. A DJ
//! scanning a track list is not reading "8A" and "9A" and comparing them; they
//! are looking for a run of similar colours, which is far faster. So the colour
//! is derived from the key's position on the circle of fifths — where adjacent
//! positions are the keys that mix — rather than from its letter name, where
//! neighbours are a semitone apart and sound terrible together.
//!
//! Twelve hues evenly around the wheel, one per position. Relative major and
//! minor share a position and so share a hue, which is right: they share a key
//! signature and mix freely. The minor is drawn slightly darker so the two are
//! still distinguishable at a glance.
//!
//! This is the scheme Mixxx's key colouring is built on rather than a copy of
//! its exact palette, which is compiled into the binary as integers.

/// The colour for a Camelot key such as `"8A"`, as `(r, g, b)`.
///
/// `None` for anything that is not a key — an empty string, or a value from a
/// source that spells keys differently. Callers draw nothing for it, which is
/// the honest rendering of "not known".
pub fn rgb_for(camelot: &str) -> Option<(u8, u8, u8)> {
    let (number, mode) = parse(camelot)?;

    // Position on the wheel, 0..11, mapped straight to a hue. Number 1 lands at
    // red and each step round the circle of fifths advances 30 degrees.
    let hue = ((number - 1) as f32) * 30.0;
    // Minor keys darker, so a glance separates 8A from 8B without reading them.
    let value = if mode == Mode::Minor { 0.72 } else { 0.92 };

    Some(hsv_to_rgb(hue, 0.78, value))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Major,
    Minor,
}

/// A Camelot key as its number and mode, if it is one.
fn parse(camelot: &str) -> Option<(u8, Mode)> {
    let camelot = camelot.trim();
    if camelot.len() < 2 {
        return None;
    }
    let (number, letter) = camelot.split_at(camelot.len() - 1);
    let mode = match letter {
        "A" | "a" => Mode::Minor,
        "B" | "b" => Mode::Major,
        _ => return None,
    };
    let number: u8 = number.parse().ok()?;
    if !(1..=12).contains(&number) {
        return None;
    }
    Some((number, mode))
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

/// The colour as `#rrggbb`, for clients that want a string.
pub fn hex_for(camelot: &str) -> Option<String> {
    let (r, g, b) = rgb_for(camelot)?;
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_a_colour() {
        for number in 1..=12 {
            for mode in ["A", "B"] {
                assert!(
                    rgb_for(&format!("{number}{mode}")).is_some(),
                    "{number}{mode}"
                );
            }
        }
    }

    /// Anything that is not a key gets no colour, so nothing is drawn for it.
    #[test]
    fn a_non_key_has_no_colour() {
        for input in ["", "A", "13A", "0A", "8C", "8", "eight-A", "  "] {
            assert_eq!(rgb_for(input), None, "{input:?} should not be a key");
        }
    }

    /// The whole point: keys that mix look alike, and keys that clash do not.
    ///
    /// 8A and 9A are neighbours on the wheel and mix; 8A and 2A are opposite it
    /// and do not. The first pair must be closer in colour than the second.
    #[test]
    fn keys_that_mix_look_alike() {
        let distance = |a: &str, b: &str| {
            let (r1, g1, b1) = rgb_for(a).unwrap();
            let (r2, g2, b2) = rgb_for(b).unwrap();
            let d = |x: u8, y: u8| (x as i32 - y as i32).pow(2);
            d(r1, r2) + d(g1, g2) + d(b1, b2)
        };

        assert!(
            distance("8A", "9A") < distance("8A", "2A"),
            "a neighbouring key should look more like 8A than the opposite one"
        );
        assert!(distance("8A", "7A") < distance("8A", "2A"));
    }

    /// Relative major and minor share a wheel position, so they share a hue —
    /// but not the same colour, or the two would be indistinguishable.
    #[test]
    fn relative_keys_share_a_hue_without_being_identical() {
        let minor = rgb_for("8A").unwrap();
        let major = rgb_for("8B").unwrap();
        assert_ne!(minor, major);
        // The minor is the darker of the two.
        let brightness = |(r, g, b): (u8, u8, u8)| r as u32 + g as u32 + b as u32;
        assert!(brightness(minor) < brightness(major));
    }

    /// Twelve positions, twelve distinguishable colours — a wheel that repeated
    /// itself would defeat the purpose.
    #[test]
    fn the_twelve_positions_are_all_different() {
        let colours: std::collections::HashSet<_> = (1..=12)
            .map(|n| rgb_for(&format!("{n}A")).unwrap())
            .collect();
        assert_eq!(colours.len(), 12);
    }

    #[test]
    fn hex_is_the_same_colour_written_differently() {
        assert_eq!(hex_for("8A").unwrap().len(), 7);
        assert!(hex_for("8A").unwrap().starts_with('#'));
        assert_eq!(hex_for("nonsense"), None);
    }

    /// Lower case is what a hand-written value looks like, and is still a key.
    #[test]
    fn case_does_not_matter() {
        assert_eq!(rgb_for("8a"), rgb_for("8A"));
        assert_eq!(rgb_for(" 8A "), rgb_for("8A"));
    }
}
