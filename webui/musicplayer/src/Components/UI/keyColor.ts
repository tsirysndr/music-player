/**
 * A colour per musical key.
 *
 * Keys are shown in traditional notation — `D`, `Fm` — because that is what
 * every other tool in the user's library shows. The *colour*, though, comes
 * from Camelot: position on the circle of fifths, where neighbours are the keys
 * that mix. Alphabetical neighbours are a semitone apart and sound terrible
 * together, so the letter cannot drive the colour.
 *
 * Twelve hues around the wheel. Relative major and minor share a position — they
 * share a key signature and mix freely — so they share a hue, with the minor
 * darker to keep them apart in a list.
 *
 * The same scheme as the daemon's `analysis::key`, deliberately: one key should
 * be one colour wherever it appears.
 */

/** Pitch class of each natural, counting semitones from C. */
const NATURALS: Record<string, number> = {
  C: 0,
  D: 2,
  E: 4,
  F: 5,
  G: 7,
  A: 9,
  B: 11,
};

/** Circle-of-fifths position for each chromatic root. C major is 8B. */
const FIFTHS = [8, 3, 10, 5, 12, 7, 2, 9, 4, 11, 6, 1];

type Key = { root: number; isMajor: boolean };

/** Parse traditional (`Fm`, `A#`, `Bb minor`) or Camelot (`4A`). */
export const parseKey = (value?: string | null): Key | null => {
  const text = value?.trim();
  if (!text) return null;
  return fromCamelot(text) ?? fromTraditional(text);
};

const fromCamelot = (text: string): Key | null => {
  const letter = text.slice(-1).toUpperCase();
  if (letter !== "A" && letter !== "B") return null;

  const number = Number(text.slice(0, -1));
  if (!Number.isInteger(number) || number < 1 || number > 12) return null;

  const majorRoot = FIFTHS.indexOf(number);
  if (majorRoot < 0) return null;

  const isMajor = letter === "B";
  // A minor key's tonic is a minor third below its relative major's.
  return { root: isMajor ? majorRoot : (majorRoot + 9) % 12, isMajor };
};

const fromTraditional = (text: string): Key | null => {
  const natural = NATURALS[text[0]?.toUpperCase()];
  if (natural === undefined) return null;

  let rest = text.slice(1);
  let root = natural;
  // `b` after a letter is always a flat — no notation writes a mode as a bare
  // `b`, so there is nothing to be ambiguous with.
  if (rest.startsWith("#") || rest.startsWith("♯")) {
    root = (natural + 1) % 12;
    rest = rest.slice(1);
  } else if (rest.startsWith("b") || rest.startsWith("♭")) {
    root = (natural + 11) % 12;
    rest = rest.slice(1);
  }

  const mode = rest.trim().toLowerCase();
  if (mode === "" || mode === "maj" || mode === "major") {
    return { root, isMajor: true };
  }
  if (mode === "m" || mode === "min" || mode === "minor") {
    return { root, isMajor: false };
  }
  return null;
};

/** Where a key sits on the circle of fifths, 1..12. */
const wheelPosition = ({ root, isMajor }: Key): number =>
  FIFTHS[isMajor ? root % 12 : (root + 3) % 12];

/** The colour for a key, or `null` if it is not one. */
export const keyColorFor = (value?: string | null): string | null => {
  const key = parseKey(value);
  if (!key) return null;
  // One step round the wheel is 30 degrees of hue.
  const hue = (wheelPosition(key) - 1) * 30;
  // Minor darker, so a glance separates Am from C without reading them.
  return `hsl(${hue} 68% ${key.isMajor ? 58 : 46}%)`;
};
