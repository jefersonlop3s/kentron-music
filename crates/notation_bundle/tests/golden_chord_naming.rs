//! Golden tests for localized chord naming. The tables mirror the notation table in
//! `docs/plano-mestre-v2.5.html`: if a value changes here, the plan must change too.

use notation_bundle::chord_naming::{Chord, Letter, NotationSystem, Note};

fn render(progression: &[&str], system: NotationSystem, key: Option<Note>) -> Vec<String> {
    progression
        .iter()
        .map(|s| {
            let chord: Chord = s.parse().unwrap_or_else(|e| panic!("{s:?}: {e}"));
            chord
                .render(system, key)
                .unwrap_or_else(|e| panic!("{s:?}: {e}"))
        })
        .collect()
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

const KEY_G: Note = Note::new(Letter::G, 0);
const KEY_BB: Note = Note::new(Letter::B, -1);

#[test]
fn progression_in_g() {
    let p = ["Gmaj9", "D/F#", "Em7", "Cm6"];
    assert_eq!(render(&p, NotationSystem::Common, None), strings(&p));
    assert_eq!(
        render(&p, NotationSystem::Latin, None),
        strings(&["Sol maj9", "Re/Fa#", "Mi m7", "Do m6"])
    );
    assert_eq!(
        render(&p, NotationSystem::German, None),
        strings(&["Gmaj9", "D/Fis", "Em7", "Cm6"])
    );
    assert_eq!(
        render(&p, NotationSystem::Nashville, Some(KEY_G)),
        strings(&["1maj9", "5/7", "6m7", "4m6"])
    );
}

#[test]
fn same_progression_three_semitones_up_in_bb() {
    let p = ["Bbmaj9", "F/A", "Gm7", "Ebm6"];
    assert_eq!(render(&p, NotationSystem::Common, None), strings(&p));
    assert_eq!(
        render(&p, NotationSystem::Latin, None),
        strings(&["Sib maj9", "Fa/La", "Sol m7", "Mib m6"])
    );
    assert_eq!(
        render(&p, NotationSystem::German, None),
        strings(&["Bmaj9", "F/A", "Gm7", "Esm6"])
    );
    // Nashville numbers do not depend on the key: identical to the progression in G.
    assert_eq!(
        render(&p, NotationSystem::Nashville, Some(KEY_BB)),
        strings(&["1maj9", "5/7", "6m7", "4m6"])
    );
}

#[test]
fn german_distinguishes_b_from_h() {
    let cases = [
        ("B", "H"),
        ("Bb", "B"),
        ("Bbb", "Heses"),
        ("B#", "His"),
        ("Eb", "Es"),
        ("Ab", "As"),
        ("Db", "Des"),
        ("Gb", "Ges"),
        ("Ebb", "Eses"),
        ("Abb", "Ases"),
        ("F#", "Fis"),
        ("C#", "Cis"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            render(&[input], NotationSystem::German, None),
            strings(&[expected]),
            "{input}"
        );
    }
}

#[test]
fn latin_spacing_rule() {
    let cases = [
        ("B", "Si"),
        ("Bb", "Sib"),
        ("F#", "Fa#"),
        ("G7", "Sol7"),
        ("Gsus4", "Sol sus4"),
        ("C6/9", "Do6/9"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            render(&[input], NotationSystem::Latin, None),
            strings(&[expected]),
            "{input}"
        );
    }
}

#[test]
fn nashville_marks_chromatic_degrees() {
    let key_c = Some(Note::new(Letter::C, 0));
    let cases = [
        ("C", "1"),
        ("Bb", "b7"),
        ("F#", "#4"),
        ("Eb", "b3"),
        ("Ab", "b6"),
        ("D", "2"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            render(&[input], NotationSystem::Nashville, key_c),
            strings(&[expected]),
            "{input}"
        );
    }
}
