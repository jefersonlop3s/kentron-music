//! Canonical chord model and localized chord naming.
//!
//! Chords are stored in a canonical, structured form and only turned into text at the
//! edge, according to the reader's notation system (plan v2.5, i18n rule 1). The `.cho`
//! file format keeps the [`NotationSystem::Common`] notation (letters A–G).
//!
//! Rust code never returns user-facing prose (rule 2): errors expose a stable
//! `message_key()` that the Flutter layer resolves through ARB files. The `Display`
//! text is English and meant for logs only.

use std::fmt;
use std::str::FromStr;

/// Letter name of a natural note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Letter {
    const fn index(self) -> usize {
        match self {
            Self::C => 0,
            Self::D => 1,
            Self::E => 2,
            Self::F => 3,
            Self::G => 4,
            Self::A => 5,
            Self::B => 6,
        }
    }

    const fn pitch_class(self) -> i32 {
        match self {
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
            Self::A => 9,
            Self::B => 11,
        }
    }

    const fn from_char(c: char) -> Option<Self> {
        match c {
            'C' => Some(Self::C),
            'D' => Some(Self::D),
            'E' => Some(Self::E),
            'F' => Some(Self::F),
            'G' => Some(Self::G),
            'A' => Some(Self::A),
            'B' => Some(Self::B),
            _ => None,
        }
    }

    /// Returns the letter at `index` modulo 7, in the order C D E F G A B.
    const fn from_index(index: usize) -> Self {
        match index % 7 {
            0 => Self::C,
            1 => Self::D,
            2 => Self::E,
            3 => Self::F,
            4 => Self::G,
            5 => Self::A,
            _ => Self::B,
        }
    }

    const fn common(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::A => "A",
            Self::B => "B",
        }
    }

    const fn latin(self) -> &'static str {
        match self {
            Self::C => "Do",
            Self::D => "Re",
            Self::E => "Mi",
            Self::F => "Fa",
            Self::G => "Sol",
            Self::A => "La",
            Self::B => "Si",
        }
    }
}

/// How a note or chord is written for a given reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotationSystem {
    /// Letters `C D E F G A B`, with `#` and `b`. Storage format of `.cho`.
    Common,
    /// Fixed-do solfège: `Do Re Mi Fa Sol La Si`.
    Latin,
    /// German names: `H` is B natural, `B` is B flat, `Fis`, `Es`, `As`.
    German,
    /// Nashville numbers relative to a key (`1`, `5/7`, `6m7`, `b7`).
    Nashville,
}

/// A spelled note: natural letter plus 0 to 2 sharps (positive) or flats (negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Note {
    pub letter: Letter,
    pub accidental: i8,
}

impl Note {
    #[must_use]
    pub const fn new(letter: Letter, accidental: i8) -> Self {
        Self { letter, accidental }
    }

    fn pitch_class(self) -> i32 {
        (self.letter.pitch_class() + i32::from(self.accidental)).rem_euclid(12)
    }

    /// Writes the note in `system`.
    ///
    /// `key` is the tonic of the major scale used as reference by
    /// [`NotationSystem::Nashville`]; it is ignored by the other systems. For songs in a
    /// minor key, pass the tonic of the relative major.
    ///
    /// # Errors
    ///
    /// [`RenderError::KeyRequired`] when `system` is Nashville and `key` is `None`.
    pub fn render(self, system: NotationSystem, key: Option<Self>) -> Result<String, RenderError> {
        Ok(match system {
            NotationSystem::Common => self.to_string(),
            NotationSystem::Latin => {
                format!(
                    "{}{}",
                    self.letter.latin(),
                    accidental_symbols(self.accidental)
                )
            }
            NotationSystem::German => self.german(),
            NotationSystem::Nashville => self.nashville(key.ok_or(RenderError::KeyRequired)?),
        })
    }

    fn german(self) -> String {
        let base = self.letter.common();
        let count = usize::from(self.accidental.unsigned_abs());
        let sharps = self.accidental > 0;
        match (self.letter, self.accidental) {
            (Letter::B, 0) => "H".to_owned(),
            (Letter::B, -1) => "B".to_owned(),
            (Letter::B, _) => format!("H{}", german_accidental(sharps, count)),
            (Letter::E | Letter::A, -1) => format!("{base}s"),
            (Letter::E | Letter::A, -2) => format!("{base}ses"),
            _ => format!("{base}{}", german_accidental(sharps, count)),
        }
    }

    fn nashville(self, tonic: Self) -> String {
        const MAJOR_STEPS: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
        let steps = (self.letter.index() + 7 - tonic.letter.index()) % 7;
        let expected = (tonic.pitch_class() + MAJOR_STEPS[steps]).rem_euclid(12);
        let mut diff = (self.pitch_class() - expected).rem_euclid(12);
        if diff > 6 {
            diff -= 12;
        }
        let symbol = if diff < 0 { "b" } else { "#" };
        let count = usize::try_from(diff.unsigned_abs()).unwrap_or(0);
        format!("{}{}", symbol.repeat(count), steps + 1)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.letter.common(),
            accidental_symbols(self.accidental)
        )
    }
}

fn accidental_symbols(accidental: i8) -> String {
    let symbol = if accidental > 0 { '#' } else { 'b' };
    std::iter::repeat_n(symbol, usize::from(accidental.unsigned_abs())).collect()
}

fn german_accidental(sharps: bool, count: usize) -> String {
    if count == 0 {
        String::new()
    } else if sharps {
        "is".repeat(count)
    } else {
        "es".repeat(count)
    }
}

/// A chord in canonical form: root, free-form quality/extension suffix, optional bass.
///
/// The suffix (`m7`, `maj9`, `sus4`, `6/9`…) is kept as written in the `.cho`; converting
/// suffix styles (`maj7` ↔ `Δ`) is a later, separate option.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chord {
    pub root: Note,
    pub suffix: String,
    pub bass: Option<Note>,
}

impl Chord {
    /// Writes the chord in `system`.
    ///
    /// In [`NotationSystem::Latin`], a space separates the root from a suffix that starts
    /// with a letter (`Sol maj9`, `Mi m7`), while digits attach directly (`Sol7`).
    ///
    /// # Errors
    ///
    /// [`RenderError::KeyRequired`] when `system` is Nashville and `key` is `None`.
    pub fn render(&self, system: NotationSystem, key: Option<Note>) -> Result<String, RenderError> {
        let root = self.root.render(system, key)?;
        let bass = self.bass.map(|b| b.render(system, key)).transpose()?;
        let separator = if system == NotationSystem::Latin
            && self.suffix.starts_with(|c: char| c.is_ascii_alphabetic())
        {
            " "
        } else {
            ""
        };
        let mut out = format!("{root}{separator}{}", self.suffix);
        if let Some(bass) = bass {
            out.push('/');
            out.push_str(&bass);
        }
        Ok(out)
    }

    /// Transposes the chord by the interval from `from` to `to`.
    ///
    /// The root and the bass are respelled diatonically: the letter moves by the same
    /// number of scale steps as the interval, so `C` up a fourth becomes `F`, not `E#`.
    /// When the diatonic spelling would need more than two accidentals, the nearest
    /// enharmonic spelling is used instead. The suffix is preserved.
    ///
    /// The interval is measured ascending, so transposing from `G` to `C` moves up a
    /// fourth rather than down a fifth.
    #[must_use]
    pub fn transpose(&self, from: Note, to: Note) -> Self {
        let (semitones, steps) = interval(from, to);
        Self {
            root: transpose_note(self.root, semitones, steps),
            suffix: self.suffix.clone(),
            bass: self.bass.map(|bass| transpose_note(bass, semitones, steps)),
        }
    }
}

/// Ascending interval between two notes: semitones and diatonic scale steps.
fn interval(from: Note, to: Note) -> (i32, i32) {
    let semitones = (to.pitch_class() - from.pitch_class()).rem_euclid(12);
    let steps = (to.letter.index() as i32 - from.letter.index() as i32).rem_euclid(7);
    (semitones, steps)
}

fn transpose_note(note: Note, semitones: i32, steps: i32) -> Note {
    let target = (note.pitch_class() + semitones).rem_euclid(12);
    let preferred = Letter::from_index(note.letter.index() + steps as usize);
    spell(target, preferred)
}

/// Spells `pitch_class` preferring `letter`, falling back to the nearest enharmonic
/// spelling when the preferred one would need more than two accidentals.
fn spell(pitch_class: i32, preferred: Letter) -> Note {
    if let Some(note) = fit(pitch_class, preferred) {
        return note;
    }
    const LETTERS: [Letter; 7] = [
        Letter::C,
        Letter::D,
        Letter::E,
        Letter::F,
        Letter::G,
        Letter::A,
        Letter::B,
    ];
    LETTERS
        .iter()
        .filter_map(|&letter| fit(pitch_class, letter))
        .min_by_key(|note| (note.accidental.unsigned_abs(), note.letter.index()))
        .unwrap_or(Note::new(preferred, 0))
}

fn fit(pitch_class: i32, letter: Letter) -> Option<Note> {
    let mut accidental = (pitch_class - letter.pitch_class()).rem_euclid(12);
    if accidental > 6 {
        accidental -= 12;
    }
    if accidental.unsigned_abs() > 2 {
        return None;
    }
    Some(Note::new(letter, accidental as i8))
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.root, self.suffix)?;
        if let Some(bass) = self.bass {
            write!(f, "/{bass}")?;
        }
        Ok(())
    }
}

impl FromStr for Chord {
    type Err = ParseError;

    /// Parses a chord written in [`NotationSystem::Common`].
    ///
    /// A trailing `/X` is a bass note when `X` is a note name; when it starts with a digit
    /// it belongs to the suffix (`C6/9`).
    fn from_str(input: &str) -> Result<Self, ParseError> {
        let (root, rest) = take_note(input.trim())?;
        let (suffix, bass) = match rest.rfind('/') {
            None => (rest, None),
            Some(i) => {
                let after = &rest[i + 1..];
                if after.is_empty() {
                    return Err(ParseError::EmptyBass);
                }
                if after.starts_with(|c: char| c.is_ascii_digit()) {
                    (rest, None)
                } else {
                    (&rest[..i], Some(parse_bass(after)?))
                }
            }
        };
        if suffix.chars().any(char::is_whitespace) {
            return Err(ParseError::InvalidSuffix);
        }
        Ok(Self {
            root,
            suffix: suffix.to_owned(),
            bass,
        })
    }
}

fn take_note(s: &str) -> Result<(Note, &str), ParseError> {
    let mut chars = s.chars();
    let first = chars.next().ok_or(ParseError::Empty)?;
    let letter = Letter::from_char(first).ok_or(ParseError::InvalidRoot(first))?;
    let rest = chars.as_str();
    let (sign, symbol) = match rest.chars().next() {
        Some('#') => (1_i8, '#'),
        Some('b') => (-1_i8, 'b'),
        _ => return Ok((Note::new(letter, 0), rest)),
    };
    let count = rest.chars().take_while(|&c| c == symbol).count();
    if count > 2 {
        return Err(ParseError::TooManyAccidentals);
    }
    let accidental = if count == 1 { sign } else { sign * 2 };
    // '#' and 'b' are one byte each, so `count` is also a byte offset.
    Ok((Note::new(letter, accidental), &rest[count..]))
}

fn parse_bass(s: &str) -> Result<Note, ParseError> {
    match take_note(s) {
        Ok((note, "")) => Ok(note),
        _ => Err(ParseError::InvalidBass(s.to_owned())),
    }
}

/// Why a chord string could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    InvalidRoot(char),
    TooManyAccidentals,
    EmptyBass,
    InvalidBass(String),
    InvalidSuffix,
}

impl ParseError {
    /// Stable key the UI layer maps to a localized message.
    #[must_use]
    pub const fn message_key(&self) -> &'static str {
        match self {
            Self::Empty => "chord.parse.empty",
            Self::InvalidRoot(_) => "chord.parse.invalid_root",
            Self::TooManyAccidentals => "chord.parse.too_many_accidentals",
            Self::EmptyBass => "chord.parse.empty_bass",
            Self::InvalidBass(_) => "chord.parse.invalid_bass",
            Self::InvalidSuffix => "chord.parse.invalid_suffix",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty chord"),
            Self::InvalidRoot(c) => write!(f, "invalid chord root {c:?}"),
            Self::TooManyAccidentals => write!(f, "more than two accidentals"),
            Self::EmptyBass => write!(f, "slash without a bass note"),
            Self::InvalidBass(s) => write!(f, "invalid bass note {s:?}"),
            Self::InvalidSuffix => write!(f, "chord suffix contains whitespace"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Why a chord or note could not be written in the requested system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// Nashville numbers need the key of the song.
    KeyRequired,
}

impl RenderError {
    /// Stable key the UI layer maps to a localized message.
    #[must_use]
    pub const fn message_key(self) -> &'static str {
        match self {
            Self::KeyRequired => "chord.render.key_required",
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyRequired => write!(f, "nashville numbers require a key"),
        }
    }
}

impl std::error::Error for RenderError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(s: &str) -> Chord {
        s.parse()
            .unwrap_or_else(|e| panic!("{s:?} should parse: {e}"))
    }

    #[test]
    fn parses_root_suffix_and_bass() {
        let c = chord("Bbmaj9/D");
        assert_eq!(c.root, Note::new(Letter::B, -1));
        assert_eq!(c.suffix, "maj9");
        assert_eq!(c.bass, Some(Note::new(Letter::D, 0)));
    }

    #[test]
    fn six_nine_is_a_suffix_not_a_bass() {
        let c = chord("C6/9");
        assert_eq!(c.suffix, "6/9");
        assert_eq!(c.bass, None);
        let c = chord("C6/9/E");
        assert_eq!(c.suffix, "6/9");
        assert_eq!(c.bass, Some(Note::new(Letter::E, 0)));
    }

    #[test]
    fn double_accidentals_are_accepted() {
        assert_eq!(chord("F##").root, Note::new(Letter::F, 2));
        assert_eq!(chord("Bbb").root, Note::new(Letter::B, -2));
    }

    #[test]
    fn rejects_malformed_chords_with_stable_keys() {
        let cases = [
            ("", ParseError::Empty, "chord.parse.empty"),
            (
                "H7",
                ParseError::InvalidRoot('H'),
                "chord.parse.invalid_root",
            ),
            (
                "g",
                ParseError::InvalidRoot('g'),
                "chord.parse.invalid_root",
            ),
            (
                "C###",
                ParseError::TooManyAccidentals,
                "chord.parse.too_many_accidentals",
            ),
            ("G/", ParseError::EmptyBass, "chord.parse.empty_bass"),
            (
                "G/X",
                ParseError::InvalidBass("X".into()),
                "chord.parse.invalid_bass",
            ),
            (
                "G/Bb7",
                ParseError::InvalidBass("Bb7".into()),
                "chord.parse.invalid_bass",
            ),
            (
                "G m7",
                ParseError::InvalidSuffix,
                "chord.parse.invalid_suffix",
            ),
        ];
        for (input, expected, key) in cases {
            let err = input.parse::<Chord>().expect_err(input);
            assert_eq!(err, expected, "input {input:?}");
            assert_eq!(err.message_key(), key, "input {input:?}");
        }
    }

    #[test]
    fn nashville_without_key_is_an_error() {
        let err = chord("G")
            .render(NotationSystem::Nashville, None)
            .expect_err("needs key");
        assert_eq!(err, RenderError::KeyRequired);
        assert_eq!(err.message_key(), "chord.render.key_required");
    }

    #[test]
    fn common_rendering_round_trips() {
        for s in [
            "Gmaj9", "D/F#", "Em7", "Cm6", "Bbmaj9", "F/A", "Ebm6", "C6/9", "G", "F##dim",
        ] {
            assert_eq!(chord(s).to_string(), s);
        }
    }

    #[test]
    fn transposes_up_a_major_second() {
        let from = Note::new(Letter::C, 0);
        let to = Note::new(Letter::D, 0);
        for (input, expected) in [
            ("C", "D"),
            ("G", "A"),
            ("Am", "Bm"),
            ("F", "G"),
            ("D/F#", "E/G#"),
        ] {
            assert_eq!(
                chord(input).transpose(from, to).to_string(),
                expected,
                "{input}"
            );
        }
    }

    #[test]
    fn transposes_using_the_ascending_interval() {
        let from = Note::new(Letter::G, 0);
        let to = Note::new(Letter::C, 0);
        assert_eq!(chord("G").transpose(from, to).to_string(), "C");
        assert_eq!(chord("D/F#").transpose(from, to).to_string(), "G/B");
    }

    #[test]
    fn double_accidental_keeps_the_diatonic_spelling() {
        let from = Note::new(Letter::C, 0);
        let to = Note::new(Letter::C, 1);
        // Cbb up a semitone is Cb: same letter, one flat fewer.
        assert_eq!(chord("Cbb").transpose(from, to).to_string(), "Cb");
    }

    #[test]
    fn double_accidental_falls_back_to_an_enharmonic_spelling() {
        let from = Note::new(Letter::C, 0);
        let to = Note::new(Letter::C, 1);
        // C## up a semitone would be C### (three sharps); the enharmonic D# is used instead.
        assert_eq!(chord("C##").transpose(from, to).to_string(), "D#");
    }

    #[test]
    fn transposition_round_trips() {
        let from = Note::new(Letter::C, 0);
        let to = Note::new(Letter::D, 0);
        for input in ["C", "Gmaj9", "D/F#", "Bbmaj9/D", "C6/9", "F##dim"] {
            let original = chord(input);
            let up = original.transpose(from, to);
            assert_eq!(up.transpose(to, from), original, "{input}");
        }
    }
}
