use std::path::{Path, PathBuf};

// ==== Config =================================================================

// ---- what a game may leave out, as the template marks it --------------------
// `others.rs` is in the player branches only: a file that is not there is skipped. The words
// are those `just check-combos` looks for: each names its option and nothing else in the
// template, which is why `shoulder` (the capsule is "half a shoulder width"), `first-person`
// (`look.rs` cites Bevy's example) and `F5` are not among them
pub(crate) const MULTIPLAYER: Removal = Removal {
    marker: "multiplayer",
    files: &[
        "src/multiplayer.rs",
        "src/multiplayer",
        "src/screens/gameplay/player/others.rs",
    ],
    words: &[
        "multiplayer",
        "Multiplayer",
        "lightyear",
        "others.rs",
        "find-server",
        "Find Server",
        "is_shared_game",
        "LockedProps",
    ],
};
pub(crate) const FIRST_PERSON: Removal = Removal {
    marker: "camera: first-person",
    files: &["src/screens/gameplay/player/first_person.rs"],
    words: &["first_person", "FirstPerson", "first person"],
};
pub(crate) const THIRD_PERSON: Removal = Removal {
    marker: "camera: third-person",
    files: &["src/screens/gameplay/player/third_person.rs"],
    words: &[
        "third_person",
        "ThirdPerson",
        "third person",
        "third-person",
        "CameraRays",
        "CAMERA_RAYS",
        "swap-shoulder",
    ],
};
pub(crate) const CAMERA_SWITCH: Removal = Removal {
    marker: "camera: switch",
    files: &["src/screens/gameplay/player/camera_switch.rs"],
    words: &[
        "camera_switch",
        "CameraView",
        "CAMERA_VIEWS",
        "switch-camera",
    ],
};
pub(crate) const SPLIT_SCREEN: Removal = Removal {
    marker: "split screen",
    files: &["src/screens/gameplay/player/split_screen.rs"],
    words: &["split_screen", "split screen", "GuestPad"],
};

/// Every option a game may leave out. A file's blocks are checked for all of them at once,
/// since one option's block may sit inside another's.
pub(crate) const OPTIONS: [Removal; 5] = [
    MULTIPLAYER,
    FIRST_PERSON,
    THIRD_PERSON,
    CAMERA_SWITCH,
    SPLIT_SCREEN,
];

// ==== Types ==================================================================

/// A camera a 3D game's player can have. Only the cameras the template has: the ones still to
/// come are lines of the menu in `prompts.rs`, nothing more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Camera {
    FirstPerson,
    ThirdPerson,
}

/// The player of a 3D game: its cameras, one or more (F5 switches between two), and whether a
/// second player on a gamepad can split the screen with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Player3d {
    pub(crate) cameras: Vec<Camera>,
    pub(crate) split_screen: bool,
}

/// The kind of game to start. A 3D game without a player is the level alone, and a 2D game
/// without the player, `TwoD(false)`, the room alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    Empty,
    ThreeD(Option<Player3d>),
    TwoD(bool),
}

/// Every answer, gathered before anything is written to disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Choices {
    pub(crate) name: String,
    pub(crate) title: String,
    pub(crate) folder: PathBuf,
    pub(crate) kind: Kind,
    pub(crate) multiplayer: bool,
}

/// Something of the template a game may leave out: the files that are it, and the marker that,
/// as a comment of each file's kind (`// <marker>`, `# <marker>`, `<!-- <marker> -->`), ends
/// every line outside them naming it, fences a block of such lines (`<marker>: begin` to
/// `<marker>: end`), or opens a without line (`without <marker>: <line>`) that stands in for
/// them. Deleting the files, the lines and the blocks, and turning the without lines into
/// code, leaves a game that builds without it, as the template promises. Its words are what
/// `just check-combos` finds in a game that left it out when the template forgot a marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Removal {
    pub(crate) marker: &'static str,
    pub(crate) files: &'static [&'static str],
    pub(crate) words: &'static [&'static str],
}

/// What becomes of an option in a game: kept, left out of a kind that offers it, or left out
/// of a kind that never had it. Only `LeftOut` is sure to find the option's files in the cloned
/// branch, but `NotOffered` still has markers to read: `GuestPad`, the split screen's, lives in
/// `main`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fate {
    Kept,
    LeftOut,
    NotOffered,
}

// ==== Helpers ================================================================

// ---- answers: what the questions decide -------------------------------------

/// Returns whether Cargo accepts `name` as a package name: ASCII letters, digits, `-` and `_`,
/// not starting with a digit.
pub(crate) fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && !name.starts_with(|c: char| c.is_ascii_digit())
}

/// Returns the game's title as the menu shows it, from its package name: `bevy-template`
/// becomes `Bevy Template`.
pub(crate) fn title_from_name(name: &str) -> String {
    let words = name.split(['-', '_']);
    words
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Returns where the game is created: a folder named after it, inside the one the user chose.
pub(crate) fn project_path(folder: &Path, name: &str) -> PathBuf {
    folder.join(name)
}

/// Returns the template branch a game of this kind is cloned from. A 3D game with a player
/// comes from `3d/player`, which has every camera, and `removals_for` takes out the ones not
/// chosen. A 2D game with the player comes from `2d/player`.
pub(crate) fn branch_for(kind: &Kind) -> &'static str {
    match kind {
        Kind::Empty => "main",
        Kind::ThreeD(None) => "3d/base",
        Kind::ThreeD(Some(_)) => "3d/player",
        Kind::TwoD(false) => "2d/base",
        Kind::TwoD(true) => "2d/player",
    }
}

/// Returns what the cloned branch has that the game was not given: multiplayer unless chosen,
/// in every kind; and in a 3D game with a player, each camera not chosen, F5's switch with
/// fewer than two, and the split screen unless chosen.
pub(crate) fn removals_for(choices: &Choices) -> Vec<Removal> {
    let mut removals = Vec::new();
    if !choices.multiplayer {
        removals.push(MULTIPLAYER);
    }
    if let Kind::ThreeD(Some(player)) = &choices.kind {
        if !player.cameras.contains(&Camera::FirstPerson) {
            removals.push(FIRST_PERSON);
        }
        if !player.cameras.contains(&Camera::ThirdPerson) {
            removals.push(THIRD_PERSON);
        }
        if player.cameras.len() < 2 {
            removals.push(CAMERA_SWITCH);
        }
        if !player.split_screen {
            removals.push(SPLIT_SCREEN);
        }
    }
    removals
}

/// Returns what becomes of `option` in the game `choices` make: multiplayer is offered to every
/// kind, and the cameras and the split screen only to a 3D game with a player. An option offered
/// is kept unless `removals_for` takes it out.
pub(crate) fn fate_of(choices: &Choices, option: &Removal) -> Fate {
    let offered = *option == MULTIPLAYER || matches!(choices.kind, Kind::ThreeD(Some(_)));
    if !offered {
        Fate::NotOffered
    } else if removals_for(choices).contains(option) {
        Fate::LeftOut
    } else {
        Fate::Kept
    }
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hyphenated_name_is_accepted() {
        assert!(is_valid_name("bevy-template"));
    }

    #[test]
    fn digits_after_the_first_character_are_accepted() {
        assert!(is_valid_name("my_game2"));
    }

    #[test]
    fn name_with_a_space_is_refused() {
        assert!(!is_valid_name("meu jogo"));
    }

    #[test]
    fn name_starting_with_a_digit_is_refused() {
        assert!(!is_valid_name("2048"));
    }

    #[test]
    fn name_with_an_accent_is_refused() {
        assert!(!is_valid_name("café"));
    }

    #[test]
    fn empty_name_is_refused() {
        assert!(!is_valid_name(""));
    }

    #[test]
    fn hyphens_split_the_title_into_capitalized_words() {
        assert_eq!(title_from_name("bevy-template"), "Bevy Template");
    }

    #[test]
    fn underscores_split_the_title_like_hyphens() {
        assert_eq!(title_from_name("my_game"), "My Game");
    }

    #[test]
    fn doubled_separators_leave_no_empty_word() {
        assert_eq!(title_from_name("my--game"), "My Game");
    }

    #[test]
    fn one_word_name_is_only_capitalized() {
        assert_eq!(title_from_name("snake"), "Snake");
    }

    #[test]
    fn game_is_created_in_a_folder_named_after_it() {
        assert_eq!(
            project_path(Path::new("games"), "snake"),
            PathBuf::from("games/snake")
        );
    }

    #[test]
    fn default_folder_puts_the_game_beside_where_the_command_runs() {
        assert_eq!(
            project_path(Path::new("."), "bevy-template"),
            PathBuf::from("./bevy-template")
        );
    }

    #[test]
    fn empty_game_starts_from_main() {
        assert_eq!(branch_for(&Kind::Empty), "main");
    }

    #[test]
    fn three_d_game_without_a_camera_starts_from_base() {
        assert_eq!(branch_for(&Kind::ThreeD(None)), "3d/base");
    }

    #[test]
    fn three_d_game_with_a_player_starts_from_player() {
        assert_eq!(
            branch_for(&Kind::ThreeD(Some(player(&[Camera::ThirdPerson], false)))),
            "3d/player"
        );
    }

    /// Returns a 3D player with these cameras, and a split screen or not.
    fn player(cameras: &[Camera], split_screen: bool) -> Player3d {
        Player3d {
            cameras: cameras.to_vec(),
            split_screen,
        }
    }

    /// Returns the choices of a game of `kind`, with multiplayer or not.
    fn choices(kind: Kind, multiplayer: bool) -> Choices {
        Choices {
            name: "game".to_string(),
            title: "Game".to_string(),
            folder: PathBuf::from("."),
            kind,
            multiplayer,
        }
    }

    #[test]
    fn a_game_with_everything_removes_nothing() {
        let both = player(&[Camera::FirstPerson, Camera::ThirdPerson], true);
        assert!(removals_for(&choices(Kind::ThreeD(Some(both)), true)).is_empty());
    }

    #[test]
    fn a_game_without_multiplayer_loses_it_whatever_its_kind() {
        for kind in [
            Kind::Empty,
            Kind::ThreeD(None),
            Kind::TwoD(false),
            Kind::TwoD(true),
        ] {
            assert_eq!(removals_for(&choices(kind, false)), vec![MULTIPLAYER]);
        }
    }

    #[test]
    fn one_camera_loses_the_other_and_the_switch() {
        let first = player(&[Camera::FirstPerson], true);
        assert_eq!(
            removals_for(&choices(Kind::ThreeD(Some(first)), true)),
            vec![THIRD_PERSON, CAMERA_SWITCH]
        );
    }

    #[test]
    fn a_player_without_a_split_screen_loses_it() {
        let both = player(&[Camera::FirstPerson, Camera::ThirdPerson], false);
        assert_eq!(
            removals_for(&choices(Kind::ThreeD(Some(both)), true)),
            vec![SPLIT_SCREEN]
        );
    }

    #[test]
    fn a_chosen_option_is_kept() {
        let first = player(&[Camera::FirstPerson], true);
        assert_eq!(
            fate_of(&choices(Kind::ThreeD(Some(first)), false), &SPLIT_SCREEN),
            Fate::Kept
        );
    }

    #[test]
    fn an_option_offered_and_not_chosen_is_left_out() {
        let first = player(&[Camera::FirstPerson], true);
        assert_eq!(
            fate_of(&choices(Kind::ThreeD(Some(first)), false), &THIRD_PERSON),
            Fate::LeftOut
        );
    }

    #[test]
    fn a_two_d_game_is_not_offered_the_split_screen() {
        assert_eq!(
            fate_of(&choices(Kind::TwoD(true), true), &SPLIT_SCREEN),
            Fate::NotOffered
        );
    }

    #[test]
    fn multiplayer_is_offered_to_an_empty_game() {
        assert_eq!(
            fate_of(&choices(Kind::Empty, false), &MULTIPLAYER),
            Fate::LeftOut
        );
    }

    #[test]
    fn no_marker_and_a_colon_start_another() {
        // `// without camera: switch: …` would otherwise be read as a line of a marker `camera`
        for option in OPTIONS {
            let start = format!("{}: ", option.marker);
            assert!(
                OPTIONS
                    .iter()
                    .all(|other| !other.marker.starts_with(&start)),
                "`{start}` starts another marker"
            );
        }
    }

    #[test]
    fn two_d_game_without_the_player_starts_from_base() {
        assert_eq!(branch_for(&Kind::TwoD(false)), "2d/base");
    }

    #[test]
    fn two_d_game_with_the_player_starts_from_player() {
        assert_eq!(branch_for(&Kind::TwoD(true)), "2d/player");
    }
}
