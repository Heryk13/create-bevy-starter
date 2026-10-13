use std::path::{Path, PathBuf};

// ==== Types ==================================================================

/// The kind of game to start. 2D has no variant: the menu shows it as "coming soon", and a
/// kind the CLI cannot build should not be a value it can hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    Empty,
    ThreeD,
}

/// Every answer, gathered before anything is written to disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Choices {
    pub(crate) name: String,
    pub(crate) title: String,
    pub(crate) folder: PathBuf,
    pub(crate) kind: Kind,
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

/// Returns where the game is created: a folder named after it, inside the one the user chose.
pub(crate) fn project_path(folder: &Path, name: &str) -> PathBuf {
    folder.join(name)
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

/// Returns the template branch a game of this kind is cloned from. 3D is the level alone until
/// the cameras in `3d/player` can be chosen.
pub(crate) fn branch_for(kind: &Kind) -> &'static str {
    match kind {
        Kind::Empty => "main",
        Kind::ThreeD => "3d/base",
    }
}

// ---- file edits: text in, text out ------------------------------------------

/// Returns `text` with every line that starts with `prefix` replaced by `new_line`, or `None`
/// when no line does: the template moved what the CLI edits, and a game made anyway would be
/// half renamed.
pub(crate) fn replace_line(text: &str, prefix: &str, new_line: &str) -> Option<String> {
    if !text.lines().any(|line| line.starts_with(prefix)) {
        None
    } else {
        let mut new_text = text
            .lines()
            .map(|line| {
                if line.starts_with(prefix) {
                    new_line
                } else {
                    line
                }
            })
            .collect::<Vec<&str>>()
            .join("\n");
        if text.ends_with('\n') {
            new_text.push('\n');
        }
        Some(new_text)
    }
}

/// Returns `text` without the section that starts at the line `heading` and runs to the next
/// `## ` heading or the end, or `None` when no line is `heading`: the template renamed the
/// section, and a game made anyway would keep it.
pub(crate) fn remove_section(text: &str, heading: &str) -> Option<String> {
    if !text.lines().any(|line| line == heading) {
        None
    } else {
        let mut inside = false;
        let mut new_text = text
            .lines()
            .filter(|line| {
                if *line == heading {
                    inside = true;
                } else if line.starts_with("## ") {
                    inside = false;
                }
                !inside
            })
            .collect::<Vec<&str>>()
            .join("\n");
        if text.ends_with('\n') {
            new_text.push('\n');
        }
        Some(new_text)
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
    fn empty_game_starts_from_main() {
        assert_eq!(branch_for(&Kind::Empty), "main");
    }

    #[test]
    fn three_d_game_starts_from_base() {
        assert_eq!(branch_for(&Kind::ThreeD), "3d/base");
    }

    #[test]
    fn only_the_line_with_the_prefix_changes() {
        assert_eq!(
            replace_line(
                "[package]\nname = \"bevy-starter\"\nversion = \"1.0.0\"",
                "name = ",
                "name = \"snake\""
            ),
            Some("[package]\nname = \"snake\"\nversion = \"1.0.0\"".to_string())
        );
    }

    #[test]
    fn text_without_the_prefix_gives_nothing() {
        assert_eq!(
            replace_line(
                "[package]\nversion = \"1.0.0\"\n",
                "name = ",
                "name = \"snake\""
            ),
            None
        );
    }

    #[test]
    fn final_line_break_is_kept() {
        assert_eq!(
            replace_line("name = \"bevy-starter\"\n", "name = ", "name = \"snake\""),
            Some("name = \"snake\"\n".to_string())
        );
    }

    #[test]
    fn section_is_removed_up_to_the_next_heading() {
        assert_eq!(
            remove_section("## A\na\n\n## Branches\nb\n\n## C\nc\n", "## Branches"),
            Some("## A\na\n\n## C\nc\n".to_string())
        );
    }

    #[test]
    fn last_section_is_removed_to_the_end() {
        assert_eq!(
            remove_section("## A\na\n## Branches\nb\n", "## Branches"),
            Some("## A\na\n".to_string())
        );
    }

    #[test]
    fn subheadings_stay_inside_the_section() {
        assert_eq!(
            remove_section("## Branches\nb\n### Detail\nd\n## C\nc\n", "## Branches"),
            Some("## C\nc\n".to_string())
        );
    }

    #[test]
    fn text_without_the_heading_gives_nothing() {
        assert_eq!(remove_section("## A\na\n", "## Branches"), None);
    }

    #[test]
    fn heading_that_only_starts_alike_gives_nothing() {
        assert_eq!(
            remove_section("## Branches and more\nb\n", "## Branches"),
            None
        );
    }
}
