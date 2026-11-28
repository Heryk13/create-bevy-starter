// ==== Types ==================================================================

/// Which end of a block a line is: the one that opens it, or the one that closes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fence {
    Begin,
    End,
}

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

/// Tells whether every block in `text`, of any of `markers`, is whole: each `begin` closed by
/// its own `end`, after the blocks opened inside it. A block left open, an `end` with nothing to
/// close, and two blocks that cross are not: no removal could tell where they end, and removing
/// one of two that cross would cut the other in half.
pub(crate) fn are_blocks_whole(text: &str, markers: &[&str]) -> bool {
    let mut open: Vec<&str> = Vec::new();
    for line in text.lines() {
        for &marker in markers {
            match fence_of(line, marker) {
                Some(Fence::Begin) => open.push(marker),
                // the guard closes the innermost block open, whichever it was
                Some(Fence::End) if open.pop() != Some(marker) => return false,
                Some(Fence::End) | None => {}
            }
        }
    }
    open.is_empty()
}

/// Returns `text` without the blocks of `marker`, each from its `// marker: begin` line to its
/// `// marker: end` line (`#` in TOML), both included, and whether any was dropped. Whatever is
/// inside goes with it, blocks of its own marker or of another included. Asked only of a text
/// `are_blocks_whole` passed: here a block left open would run to the end.
pub(crate) fn remove_marked_blocks(text: &str, marker: &str) -> (String, bool) {
    let mut depth = 0_usize;
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| match fence_of(line, marker) {
            Some(Fence::Begin) => {
                depth += 1;
                false
            }
            Some(Fence::End) => {
                depth = depth.saturating_sub(1);
                false
            }
            None => depth == 0,
        })
        .collect();
    let dropped = kept.len() != text.lines().count();
    let mut new_text = kept.join("\n");
    if text.ends_with('\n') && !new_text.is_empty() {
        new_text.push('\n');
    }
    (new_text, dropped)
}

/// Returns `text` without the lines that end with `marker` as a comment, `// marker` in Rust or
/// `# marker` in TOML, and whether any was dropped. The comment is matched whole, so
/// `camera: first-person` never takes a line marked `camera: first`.
pub(crate) fn remove_marked_lines(text: &str, marker: &str) -> (String, bool) {
    let rust = format!("// {marker}");
    let toml = format!("# {marker}");
    let marked = |line: &&str| {
        let line = line.trim_end();
        line.ends_with(&rust) || line.ends_with(&toml)
    };
    let kept: Vec<&str> = text.lines().filter(|line| !marked(line)).collect();
    let dropped = kept.len() != text.lines().count();
    let mut new_text = kept.join("\n");
    if text.ends_with('\n') && !new_text.is_empty() {
        new_text.push('\n');
    }
    (new_text, dropped)
}

/// Returns `text` with every without line of `marker` (`// without marker: <line>`, `#` in
/// TOML) turned into the line it holds, at the comment's indentation, and whether any was. A
/// game that leaves the option out gets the line that stands in for what went.
pub(crate) fn restore_without_lines(text: &str, marker: &str) -> (String, bool) {
    let mut restored = false;
    let lines: Vec<String> = text
        .lines()
        .map(|line| match without_line_of(line, marker) {
            Some(code) => {
                restored = true;
                code
            }
            None => line.to_string(),
        })
        .collect();
    let mut new_text = lines.join("\n");
    if text.ends_with('\n') && !new_text.is_empty() {
        new_text.push('\n');
    }
    (new_text, restored)
}

/// Returns `text` without the without lines of `marker`, and whether any was dropped. A game
/// that keeps the option will never use them, and commented-out code is only noise.
pub(crate) fn remove_without_lines(text: &str, marker: &str) -> (String, bool) {
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| without_line_of(line, marker).is_none())
        .collect();
    let dropped = kept.len() != text.lines().count();
    let mut new_text = kept.join("\n");
    if text.ends_with('\n') && !new_text.is_empty() {
        new_text.push('\n');
    }
    (new_text, dropped)
}

/// Returns the line a without line of `marker` holds, at the comment's indentation, if `line`
/// is one. Matched whole: the marker must be followed by `: `, so `camera: first` never takes
/// `// without camera: first-person: …`, which is why no marker and `: ` may start another.
fn without_line_of(line: &str, marker: &str) -> Option<String> {
    let comment = line.trim_start();
    let indent = &line[..line.len() - comment.len()];
    let code = comment
        .strip_prefix("// ")
        .or_else(|| comment.strip_prefix("# "))?
        .strip_prefix("without ")?
        .strip_prefix(marker)?
        .strip_prefix(": ")?;
    Some(format!("{indent}{code}"))
}

/// Returns which fence of `marker`'s blocks `line` is, if it is one: the whole line, spaces
/// aside, is `// marker: begin` or `// marker: end`, or the same with `#`. Matched whole, so
/// `camera: first` never opens a block of `camera: first-person`.
fn fence_of(line: &str, marker: &str) -> Option<Fence> {
    let line = line.trim();
    let comment = line
        .strip_prefix("// ")
        .or_else(|| line.strip_prefix("# "))?;
    match comment.strip_prefix(marker)?.strip_prefix(": ")? {
        "begin" => Some(Fence::Begin),
        "end" => Some(Fence::End),
        _ => None,
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

    #[test]
    fn a_block_inside_another_is_whole() {
        let text = "// multiplayer: begin\n// split screen: begin\nx\n// split screen: end\n\
                    // multiplayer: end\n";
        assert!(are_blocks_whole(text, &["multiplayer", "split screen"]));
    }

    #[test]
    fn a_block_never_closed_is_broken() {
        let text = "a\n// multiplayer: begin\nx\n";
        assert!(!are_blocks_whole(text, &["multiplayer"]));
    }

    #[test]
    fn an_end_with_nothing_to_close_is_broken() {
        let text = "x\n// multiplayer: end\na\n";
        assert!(!are_blocks_whole(text, &["multiplayer"]));
    }

    #[test]
    fn blocks_that_cross_are_broken() {
        let text = "// multiplayer: begin\n// split screen: begin\n// multiplayer: end\n\
                    // split screen: end\n";
        assert!(!are_blocks_whole(text, &["multiplayer", "split screen"]));
    }

    #[test]
    fn a_marked_block_goes_with_its_fences() {
        let text = "a\n    // split screen: begin\n    x\n    // split screen: end\nb\n";
        assert_eq!(
            remove_marked_blocks(text, "split screen"),
            ("a\nb\n".to_string(), true)
        );
    }

    #[test]
    fn a_toml_block_goes_like_a_rust_one() {
        let text =
            "[dependencies]\n# multiplayer: begin\nlightyear = \"0.31\"\n# multiplayer: end\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer"),
            ("[dependencies]\n".to_string(), true)
        );
    }

    #[test]
    fn a_block_inside_one_of_its_own_marker_goes_with_it() {
        let text = "// multiplayer: begin\na\n// multiplayer: begin\nb\n// multiplayer: end\nc\n\
                    // multiplayer: end\nd\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer"),
            ("d\n".to_string(), true)
        );
    }

    #[test]
    fn a_block_takes_the_blocks_of_other_markers_inside_it() {
        let text = "// multiplayer: begin\n// split screen: begin\nx\n// split screen: end\n\
                    // multiplayer: end\nd\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer"),
            ("d\n".to_string(), true)
        );
    }

    #[test]
    fn a_block_of_another_marker_stays() {
        let text = "// split screen: begin\nx\n// split screen: end\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer"),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_fence_is_matched_whole() {
        let text = "// camera: first-person: begin\nx\n// camera: first-person: end\n";
        assert_eq!(
            remove_marked_blocks(text, "camera: first"),
            (text.to_string(), false)
        );
    }

    #[test]
    fn marked_lines_go_and_the_rest_stays() {
        let text = "mod a; // multiplayer\nmod b;\nlightyear = \"0.31\" # multiplayer\n";
        assert_eq!(
            remove_marked_lines(text, "multiplayer"),
            ("mod b;\n".to_string(), true)
        );
    }

    #[test]
    fn a_marker_is_matched_whole() {
        let text = "mod first_person; // camera: first-person\n";
        assert_eq!(
            remove_marked_lines(text, "camera: first"),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_without_line_becomes_its_line_at_its_indentation() {
        let text = "struct A {\n    // without split screen: pads: Query<&Gamepad>,\n}\n";
        assert_eq!(
            restore_without_lines(text, "split screen"),
            (
                "struct A {\n    pads: Query<&Gamepad>,\n}\n".to_string(),
                true
            )
        );
    }

    #[test]
    fn a_toml_without_line_becomes_its_line() {
        let text = "# without multiplayer: name = \"solo\"\n";
        assert_eq!(
            restore_without_lines(text, "multiplayer"),
            ("name = \"solo\"\n".to_string(), true)
        );
    }

    #[test]
    fn a_without_line_of_another_marker_stays_a_comment() {
        let text = "// without multiplayer: mod files;\n";
        assert_eq!(
            restore_without_lines(text, "split screen"),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_without_line_is_matched_whole() {
        let text = "// without camera: first-person: mod a;\n";
        assert_eq!(
            restore_without_lines(text, "camera: first"),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_kept_option_loses_its_without_lines() {
        let text = "a\n    // without split screen: b\nc\n";
        assert_eq!(
            remove_without_lines(text, "split screen"),
            ("a\nc\n".to_string(), true)
        );
    }
}
