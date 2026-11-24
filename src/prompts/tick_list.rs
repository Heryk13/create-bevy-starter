use crossterm::cursor::{Hide, MoveToColumn, MoveUp, Show};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read};
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use crossterm::{QueueableCommand, queue};
use inquire::InquireError;
use inquire::error::InquireResult;
use std::fmt::{self, Display, Formatter};
use std::io::{Stderr, Write, stderr};

// ==== Config =================================================================

// ---- words ------------------------------------------------------------------
const DONE_WORDS: &str = "Done";
const TICK_HELP: &str = "[↑↓ to move, Enter or Space to tick, Done to go on]";
// what the answer says when none is ticked
const NONE_TICKED_WORDS: &str = "None";

// ---- the list drawn: the question, a line each, Done and the help -----------
// the lines that are not an item: the question, Done and the help
const LINES_AROUND_ITEMS: usize = 3;

// ==== Ticking ================================================================

/// Asks for any of `items`, Enter or Space on one ticking or unticking it, until Done, the last
/// line, goes on; Esc gives the question up and Ctrl+C stops the CLI, as in inquire's own
/// questions. Neither key alone goes on to the next question: inquire's own list ticks with
/// Space and answers on Enter, and people pressed Enter to tick and skipped the question without
/// noticing. Drawn on stderr, where inquire draws, in its colors. Returns the ones ticked, in
/// the order of `items`.
pub(super) fn ask_ticks<T: Copy + Display>(message: &str, items: &[T]) -> InquireResult<Vec<T>> {
    let mut out = stderr();
    let _raw = RawMode::start()?;
    let mut ticked = vec![false; items.len()];
    // the line the cursor is on: an item, or `items.len()` for Done
    let mut cursor = 0;
    draw(&mut out, message, items, &ticked, cursor, false)?;
    loop {
        let Event::Key(key) = read()? else {
            continue;
        };
        let Some(action) = key_action(key) else {
            continue;
        };
        match action {
            KeyAction::Up => cursor = cursor.saturating_sub(1),
            KeyAction::Down => cursor = (cursor + 1).min(items.len()),
            KeyAction::Press if cursor == items.len() => {
                let answer = ticked_summary(items, &ticked);
                erase(&mut out, items.len())?;
                write!(out, "{} {message} {}\r\n", "?".green(), answer.cyan())?;
                out.flush()?;
                return Ok(ticked_items(items, &ticked));
            }
            KeyAction::Press => {
                if let Some(tick) = ticked.get_mut(cursor) {
                    *tick = !*tick;
                }
            }
            KeyAction::Cancel => {
                erase(&mut out, items.len())?;
                write!(out, "{} {message} {}\r\n", "?".green(), "<canceled>".red())?;
                out.flush()?;
                return Err(InquireError::OperationCanceled);
            }
            KeyAction::Interrupt => {
                erase(&mut out, items.len())?;
                out.flush()?;
                return Err(InquireError::OperationInterrupted);
            }
        }
        draw(&mut out, message, items, &ticked, cursor, true)?;
    }
}

// ==== Types ==================================================================

/// What a key does in the list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum KeyAction {
    Up,
    Down,
    Press,
    Cancel,
    Interrupt,
}

/// One line of the list: an item, ticked or not, or Done, which goes on.
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

/// The terminal in raw mode, where every key comes as it is pressed, and the cursor hidden,
/// until it is dropped: an error or a panic mid-question still gives the terminal back.
struct RawMode;

impl RawMode {
    fn start() -> InquireResult<Self> {
        enable_raw_mode()?;
        stderr().queue(Hide)?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // nothing to do with an error here: the terminal is as good as it gets
        let _ = stderr().queue(Show).and_then(Write::flush);
        let _ = disable_raw_mode();
    }
}

// ==== Helpers ================================================================

/// Draws the list: the question, the lines, the cursor's marked, Done and the help. With
/// `again`, over the list drawn before, which is as many lines.
fn draw<T: Copy + Display>(
    out: &mut Stderr,
    message: &str,
    items: &[T],
    ticked: &[bool],
    cursor: usize,
    again: bool,
) -> InquireResult<()> {
    if again {
        erase(out, items.len())?;
    }
    write!(out, "{} {message}\r\n", "?".green())?;
    for (index, line) in tick_lines(items, ticked).iter().enumerate() {
        if index == cursor {
            write!(out, "{} {}\r\n", ">".cyan(), line.to_string().cyan())?;
        } else {
            write!(out, "  {line}\r\n")?;
        }
    }
    write!(out, "{}", TICK_HELP.dark_cyan())?;
    out.flush()?;
    Ok(())
}

/// Clears the list drawn for `item_count` items, leaving the cursor where it started.
fn erase(out: &mut Stderr, item_count: usize) -> InquireResult<()> {
    // the cursor is on the help, the last line: up to the question, then everything below
    let up = u16::try_from(item_count + LINES_AROUND_ITEMS - 1).unwrap_or(u16::MAX);
    queue!(
        out,
        MoveToColumn(0),
        MoveUp(up),
        Clear(ClearType::FromCursorDown)
    )?;
    Ok(())
}

/// Returns what a key does in the list, or `None` when it does nothing. Only presses count:
/// Windows reports each key's release too.
fn key_action(key: KeyEvent) -> Option<KeyAction> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    match (key.code, key.modifiers) {
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(KeyAction::Interrupt),
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(KeyAction::Up),
        (KeyCode::Down | KeyCode::Char('j'), _) => Some(KeyAction::Down),
        (KeyCode::Enter | KeyCode::Char(' '), _) => Some(KeyAction::Press),
        (KeyCode::Esc, _) => Some(KeyAction::Cancel),
        _ => None,
    }
}

/// Returns the lines of the list: each item with whether it is ticked, then Done.
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

/// Returns what the answer says: the items ticked, or `None`.
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
    use crossterm::event::KeyEventState;

    const ITEMS: [&str; 2] = ["Split screen", "Multiplayer"];

    fn pressed(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn enter_and_space_both_tick() {
        assert_eq!(key_action(pressed(KeyCode::Enter)), Some(KeyAction::Press));
        assert_eq!(
            key_action(pressed(KeyCode::Char(' '))),
            Some(KeyAction::Press)
        );
    }

    #[test]
    fn a_key_let_go_does_nothing() {
        let released = KeyEvent {
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
            ..pressed(KeyCode::Enter)
        };
        assert_eq!(key_action(released), None);
    }

    #[test]
    fn esc_gives_up_and_ctrl_c_stops() {
        assert_eq!(key_action(pressed(KeyCode::Esc)), Some(KeyAction::Cancel));
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(key_action(ctrl_c), Some(KeyAction::Interrupt));
    }

    #[test]
    fn a_list_shows_each_tick_and_ends_with_done() {
        let lines: Vec<String> = tick_lines(&ITEMS, &[true, false])
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(lines, ["[x] Split screen", "[ ] Multiplayer", DONE_WORDS]);
    }

    #[test]
    fn only_the_ticked_items_come_back_in_their_order() {
        assert_eq!(ticked_items(&ITEMS, &[false, true]), ["Multiplayer"]);
        assert!(ticked_items(&ITEMS, &[false, false]).is_empty());
    }

    #[test]
    fn the_answer_names_the_ticked_items_or_none() {
        assert_eq!(ticked_summary(&ITEMS, &[false, false]), NONE_TICKED_WORDS);
        assert_eq!(
            ticked_summary(&ITEMS, &[true, true]),
            "Split screen, Multiplayer"
        );
    }
}
