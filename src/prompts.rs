use inquire::{Confirm, MultiSelect, Select, Text, error::InquireResult, validator::Validation};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use crate::choices::{Camera, Choices, Kind, is_valid_name, project_path, title_from_name};

// ==== Config =================================================================

/// What makes a project name valid, said the same way to the question and to the argument.
pub(crate) const NAME_RULE: &str = "Use ASCII letters, digits, - and _, not starting with a digit.";

// ==== Types ==================================================================

// CONTRIBUTING gives a choice the CLI cannot build no variant, and `KindItem::TwoD` and
// `MultiplayerItem::Yes` are such choices: the menus have to show them as coming soon. They
// never leave this file: their question asks again when one is picked, and hands back only
// what the CLI can build

/// One line of the kind menu.
#[derive(Clone, Copy)]
enum KindItem {
    Empty,
    ThreeD,
    TwoD,
}

impl Display for KindItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let label = match self {
            KindItem::Empty => "Empty",
            KindItem::ThreeD => "3D",
            KindItem::TwoD => "2D (coming soon)",
        };
        write!(f, "{label}")
    }
}

/// One line of the multiplayer menu.
#[derive(Clone, Copy)]
enum MultiplayerItem {
    No,
    Yes,
}

impl Display for MultiplayerItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let label = match self {
            MultiplayerItem::No => "No",
            MultiplayerItem::Yes => "Yes (coming soon)",
        };
        write!(f, "{label}")
    }
}

// the menu's words for a camera live with the questions, so `choices.rs` never decides how
// anything looks on screen
impl Display for Camera {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let label = match self {
            Camera::FirstPerson => "First-person",
            Camera::ThirdPerson => "Third-person",
            Camera::Isometric => "Isometric",
        };
        write!(f, "{label}")
    }
}

// ==== Questions ==============================================================

/// Asks every question in order and gathers the answers, before anything is written to disk.
/// A `name` or `folder` given as an argument is used as it is, and its question is skipped.
/// Ctrl+C at any of them comes back as an error, for `main` to decide what it means.
pub(crate) fn ask_choices(name: Option<String>, folder: Option<PathBuf>) -> InquireResult<Choices> {
    let name = match name {
        Some(name) => name,
        None => ask_name()?,
    };
    let folder = match folder {
        Some(folder) => folder,
        None => ask_folder()?,
    };
    let title = ask_title(&name)?;
    let kind = ask_kind()?;
    ask_multiplayer()?;
    Ok(Choices {
        name,
        title,
        folder,
        kind,
    })
}

/// Asks for the project name, and asks again until Cargo would accept it.
fn ask_name() -> InquireResult<String> {
    Text::new("Project name:")
        .with_default("bevy-template")
        .with_validator(|input: &str| {
            if is_valid_name(input) {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid(NAME_RULE.into()))
            }
        })
        .prompt()
}

/// Asks for the folder the game is created in. Any answer is accepted: whether the game's own
/// folder inside it is free can only be checked once the name is known, by `scaffold.rs`.
fn ask_folder() -> InquireResult<PathBuf> {
    Text::new("Where to create it:")
        .with_default(".")
        .prompt()
        .map(PathBuf::from)
}

/// Asks for the game's title as the menu shows it, offering the one made from `name`.
fn ask_title(name: &str) -> InquireResult<String> {
    let default = title_from_name(name);
    Text::new("Game title:").with_default(&default).prompt()
}

/// Asks what kind of game to start, and asks again when 2D is picked: it is shown, but it
/// cannot be built yet.
fn ask_kind() -> InquireResult<Kind> {
    loop {
        let item = Select::new(
            "Kind:",
            vec![KindItem::Empty, KindItem::ThreeD, KindItem::TwoD],
        )
        .prompt()?;
        match item {
            KindItem::Empty => return Ok(Kind::Empty),
            KindItem::ThreeD => {
                return Ok(Kind::ThreeD {
                    cameras: ask_cameras()?,
                });
            }
            KindItem::TwoD => println!("2D is coming soon. Pick Empty or 3D."),
        }
    }
}

/// Asks which cameras a 3D game keeps. None is a valid answer: the level alone, from
/// `3d/base`.
fn ask_cameras() -> InquireResult<Vec<Camera>> {
    MultiSelect::new(
        "Cameras (none = the level only, no player):",
        vec![Camera::FirstPerson, Camera::ThirdPerson, Camera::Isometric],
    )
    .prompt()
}

/// Asks whether the game is multiplayer, and asks again when Yes is picked: it is shown so the
/// user knows it is coming, but it cannot be built yet, so the answer changes nothing.
fn ask_multiplayer() -> InquireResult<()> {
    loop {
        let item = Select::new(
            "Multiplayer:",
            vec![MultiplayerItem::No, MultiplayerItem::Yes],
        )
        .prompt()?;
        match item {
            MultiplayerItem::No => return Ok(()),
            MultiplayerItem::Yes => println!("Multiplayer is coming soon. Pick No."),
        }
    }
}

/// Shows where the game will be created and asks to go on. Enter says yes; no leaves the disk
/// as it was.
pub(crate) fn ask_confirmation(choices: &Choices) -> InquireResult<bool> {
    let path = project_path(&choices.folder, &choices.name);
    let message = format!("Create {} in {}?", choices.name, path.display());
    Confirm::new(&message).with_default(true).prompt()
}
