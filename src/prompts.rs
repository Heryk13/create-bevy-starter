use inquire::{Confirm, Select, Text, error::InquireResult, validator::Validation};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use crate::choices::{
    Camera, Choices, Kind, Player3d, is_valid_name, project_path, title_from_name,
};
use tick_list::ask_ticks;

mod tick_list;

// ==== Config =================================================================

/// What makes a project name valid, said the same way to the question and to the argument.
pub(crate) const NAME_RULE: &str = "Use ASCII letters, digits, - and _, not starting with a digit.";

// ==== Types ==================================================================

// CONTRIBUTING gives a choice the CLI cannot build no variant, and `CameraItem::Isometric` is
// such a choice: the menu has to show it as coming soon. It never leaves this file: its question
// asks again when it is picked, and hands back only what the CLI can build

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
            KindItem::TwoD => "2D",
        };
        write!(f, "{label}")
    }
}

/// One line of the camera menu, asked of a 3D game, where any of them can be ticked.
#[derive(Clone, Copy)]
enum CameraItem {
    FirstPerson,
    ThirdPerson,
    Isometric,
}

impl Display for CameraItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let label = match self {
            CameraItem::FirstPerson => "First person",
            CameraItem::ThirdPerson => "Third person",
            CameraItem::Isometric => "Isometric (coming soon)",
        };
        write!(f, "{label}")
    }
}

/// One line of the extras menu, where any of them can be ticked.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ExtraItem {
    SplitScreen,
    Multiplayer,
}

impl Display for ExtraItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let label = match self {
            ExtraItem::SplitScreen => "Split screen (a second player on a gamepad)",
            ExtraItem::Multiplayer => "Multiplayer (co-op over the LAN, one player hosting)",
        };
        write!(f, "{label}")
    }
}

/// One line of the player menu, asked of a 2D game.
#[derive(Clone, Copy)]
enum PlayerItem {
    None,
    TopDownSquare,
}

impl Display for PlayerItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let label = match self {
            PlayerItem::None => "None",
            PlayerItem::TopDownSquare => "Top-down square",
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
    let mut kind = ask_kind()?;
    let player_3d = match &mut kind {
        Kind::ThreeD(Some(player)) => Some(player),
        _ => None,
    };
    let extras = ask_extras(player_3d.is_some())?;
    if let Some(player) = player_3d {
        player.split_screen = extras.contains(&ExtraItem::SplitScreen);
    }
    let multiplayer = extras.contains(&ExtraItem::Multiplayer);
    Ok(Choices {
        name,
        title,
        folder,
        kind,
        multiplayer,
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

/// Asks what kind of game to start, of a 3D one its player, and of a 2D one its player.
fn ask_kind() -> InquireResult<Kind> {
    let item = Select::new(
        "Kind:",
        vec![KindItem::Empty, KindItem::ThreeD, KindItem::TwoD],
    )
    .prompt()?;
    Ok(match item {
        KindItem::Empty => Kind::Empty,
        KindItem::ThreeD => Kind::ThreeD(ask_player_3d()?),
        KindItem::TwoD => Kind::TwoD(ask_player()?),
    })
}

/// Asks for a 3D game's cameras. No camera is the level alone; any is a player, whose split
/// screen `ask_extras` asks for, off until then.
fn ask_player_3d() -> InquireResult<Option<Player3d>> {
    let cameras = ask_cameras()?;
    if cameras.is_empty() {
        return Ok(None);
    }
    Ok(Some(Player3d {
        cameras,
        split_screen: false,
    }))
}

/// Asks whether a 2D game starts with a player, a square that walks the room, or with the
/// room alone.
fn ask_player() -> InquireResult<bool> {
    let item =
        Select::new("Player:", vec![PlayerItem::None, PlayerItem::TopDownSquare]).prompt()?;
    Ok(matches!(item, PlayerItem::TopDownSquare))
}

/// Asks which cameras a 3D game's player has, any of them ticked; none ticked is the level
/// alone, with no player. Two make F5 switch between them. Asks again when the one that is
/// coming soon is ticked: it is shown so the user knows it is coming, but it cannot be built
/// yet.
fn ask_cameras() -> InquireResult<Vec<Camera>> {
    loop {
        let items = ask_ticks(
            "Cameras (none for the level alone):",
            &[
                CameraItem::FirstPerson,
                CameraItem::ThirdPerson,
                CameraItem::Isometric,
            ],
        )?;
        if items
            .iter()
            .any(|item| matches!(item, CameraItem::Isometric))
        {
            println!("Isometric is coming soon. Untick it.");
            continue;
        }
        return Ok(items
            .into_iter()
            .filter_map(|item| match item {
                CameraItem::FirstPerson => Some(Camera::FirstPerson),
                CameraItem::ThirdPerson => Some(Camera::ThirdPerson),
                CameraItem::Isometric => None,
            })
            .collect());
    }
}

/// Asks for the extras, any of them ticked: split screen, offered only when `has_player_3d`,
/// since a second player on the same screen needs a 3D player to copy, and multiplayer,
/// friends playing together over the LAN or a VPN, one hosting.
fn ask_extras(has_player_3d: bool) -> InquireResult<Vec<ExtraItem>> {
    let mut items = Vec::new();
    if has_player_3d {
        items.push(ExtraItem::SplitScreen);
    }
    items.push(ExtraItem::Multiplayer);
    ask_ticks("Extras (none is fine):", &items)
}

/// Shows where the game will be created and asks to go on. Enter says yes; no leaves the disk
/// as it was.
pub(crate) fn ask_confirmation(choices: &Choices) -> InquireResult<bool> {
    let path = project_path(&choices.folder, &choices.name);
    let message = format!("Create {} in {}?", choices.name, path.display());
    Confirm::new(&message).with_default(true).prompt()
}
