// ==== Helpers ================================================================

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
