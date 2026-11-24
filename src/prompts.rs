use inquire::list_option::ListOption;
use inquire::{Confirm, Select, Text, error::InquireResult, validator::Validation};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use crate::choices::{
    Camera, Choices, Kind, Player3d, is_valid_name, project_path, title_from_name,
};

// ==== Config =================================================================

/// What makes a project name valid, said the same way to the question and to the argument.
pub(crate) const NAME_RULE: &str = "Use ASCII letters, digits, - and _, not starting with a digit.";

// ---- lists where several can be ticked: Enter ticks, Done goes on -----------
const DONE_WORDS: &str = "Done";
const TICK_HELP: &str = "↑↓ to move, Enter to tick or untick, Done to go on";
// what a tick's answer says when none is ticked
const NONE_TICKED_WORDS: &str = "None";
// up a line and clear it: the answer inquire prints for each tick, so only Done's stays
const ERASE_LINE: &str = "\x1b[1A\x1b[2K";

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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

/// One line of a list asked with `ask_ticks`: an item, ticked or not, or Done, which goes on.
#[derive(Clone, Copy)]
enum TickLine<T> {
    Item { item: T, ticked: bool },
    Done,
}

impl<T: Display> Display for TickLine<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TickLine::Item { item, ticked: true } => write!(f, "[x] {item}"),
            TickLine::Item {
                item,
                ticked: false,
            } => write!(f, "[ ] {item}"),
            TickLine::Done => write!(f, "{DONE_WORDS}"),
        }
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

/// Asks for any of `items`, each Enter on one ticking or unticking it and asking again with the
/// cursor where it was, until Done, the last line. Enter alone never goes on to the next
/// question, as it did in inquire's own list, where Space ticks and Enter answers: people
/// pressed Enter to tick and skipped the question without noticing. Returns the ones ticked,
/// in the order of `items`.
fn ask_ticks<T: Copy + Display>(message: &str, items: &[T]) -> InquireResult<Vec<T>> {
    let mut ticked = vec![false; items.len()];
    let mut cursor = 0;
    loop {
        let summary = ticked_summary(items, &ticked);
        let formatter = |_: ListOption<&TickLine<T>>| summary.clone();
        let picked = Select::new(message, tick_lines(items, &ticked))
            .with_starting_cursor(cursor)
            .with_help_message(TICK_HELP)
            .with_formatter(&formatter)
            .without_filtering()
            .raw_prompt()?;
        if let TickLine::Done = picked.value {
            return Ok(ticked_items(items, &ticked));
        }
        if let Some(tick) = ticked.get_mut(picked.index) {
            *tick = !*tick;
        }
        cursor = picked.index;
        // inquire answers on stderr, so its line is erased there
        eprint!("{ERASE_LINE}");
    }
}

/// Shows where the game will be created and asks to go on. Enter says yes; no leaves the disk
/// as it was.
pub(crate) fn ask_confirmation(choices: &Choices) -> InquireResult<bool> {
    let path = project_path(&choices.folder, &choices.name);
    let message = format!("Create {} in {}?", choices.name, path.display());
    Confirm::new(&message).with_default(true).prompt()
}

// ==== Helpers ================================================================

/// Returns the lines of a list asked with `ask_ticks`: each item with whether it is ticked,
/// then Done.
fn tick_lines<T: Copy>(items: &[T], ticked: &[bool]) -> Vec<TickLine<T>> {
    items
        .iter()
        .zip(ticked)
        .map(|(item, ticked)| TickLine::Item {
            item: *item,
            ticked: *ticked,
        })
        .chain([TickLine::Done])
        .collect()
}

/// Returns the items of `items` that `ticked` marks, in their order.
fn ticked_items<T: Copy>(items: &[T], ticked: &[bool]) -> Vec<T> {
    items
        .iter()
        .zip(ticked)
        .filter(|(_, ticked)| **ticked)
        .map(|(item, _)| *item)
        .collect()
}

/// Returns what the answer of a list says: the items ticked, or `None`.
fn ticked_summary<T: Copy + Display>(items: &[T], ticked: &[bool]) -> String {
    let names: Vec<String> = ticked_items(items, ticked)
        .iter()
        .map(ToString::to_string)
        .collect();
    if names.is_empty() {
        NONE_TICKED_WORDS.to_string()
    } else {
        names.join(", ")
    }
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const ITEMS: [ExtraItem; 2] = [ExtraItem::SplitScreen, ExtraItem::Multiplayer];

    #[test]
    fn a_list_shows_each_tick_and_ends_with_done() {
        let lines: Vec<String> = tick_lines(&ITEMS, &[true, false])
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            lines,
            [
                "[x] Split screen (a second player on a gamepad)",
                "[ ] Multiplayer (co-op over the LAN, one player hosting)",
                DONE_WORDS,
            ]
        );
    }

    #[test]
    fn only_the_ticked_items_come_back_in_their_order() {
        assert_eq!(
            ticked_items(&ITEMS, &[false, true]),
            [ExtraItem::Multiplayer]
        );
        assert!(ticked_items(&ITEMS, &[false, false]).is_empty());
    }

    #[test]
    fn the_answer_names_the_ticked_items_or_none() {
        assert_eq!(ticked_summary(&ITEMS, &[false, false]), NONE_TICKED_WORDS);
        assert_eq!(
            ticked_summary(
                &[CameraItem::FirstPerson, CameraItem::ThirdPerson],
                &[true, true]
            ),
            "First person, Third person"
        );
    }
}
