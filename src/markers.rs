// ==== Types ==================================================================

/// How a kind of file writes a comment, where its markers live: `//` in Rust, `#` in TOML and
/// Fluent, `<!-- … -->` in Markdown. One per file, from its extension (`comment_for`): in
/// Markdown a `#` starts a heading, so `# multiplayer` must never read as a marker there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Comment {
    Slashes,
    Hash,
    Html,
}

impl Comment {
    /// Returns what opens this kind of comment, the space after it included.
    fn open(self) -> &'static str {
        match self {
            Comment::Slashes => "// ",
            Comment::Hash => "# ",
            Comment::Html => "<!-- ",
        }
    }

    /// Returns what closes this kind of comment, the space before it included: nothing for the
    /// kinds that run to the end of the line.
    fn close(self) -> &'static str {
        match self {
            Comment::Slashes | Comment::Hash => "",
            Comment::Html => " -->",
        }
    }

    /// Returns `text` written as this kind of comment: `// text`, `# text` or `<!-- text -->`.
    fn around(self, text: &str) -> String {
        format!("{}{text}{}", self.open(), self.close())
    }
}

/// Which end of a block a line is: the one that opens it, or the one that closes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fence {
    Begin,
    End,
}

// ==== Helpers ================================================================

/// Returns how a file with this extension writes its markers, or `None` for a kind of file
/// that has none: `rs` takes `//`, `toml` and `ftl` take `#`, `md` takes `<!-- … -->`.
pub(crate) fn comment_for(extension: &str) -> Option<Comment> {
    match extension {
        "rs" => Some(Comment::Slashes),
        "toml" | "ftl" => Some(Comment::Hash),
        "md" => Some(Comment::Html),
        _ => None,
    }
}

/// Tells whether every marker in `text`, of any of `markers`, is whole. Each block's `begin` is
/// closed by its own `end`, after the blocks opened inside it: a block left open, an `end` with
/// nothing to close, and two blocks that cross are not whole, since no removal could tell where
/// they end, and removing one of two that cross would cut the other in half. And each without
/// line closes its comment once, at its end: in Markdown a `-->` inside it would end the comment
/// early as the template renders, and one never closed would hide the rest of the file.
pub(crate) fn are_markers_whole(text: &str, markers: &[&str], comment: Comment) -> bool {
    let mut open: Vec<&str> = Vec::new();
    for line in text.lines() {
        for &marker in markers {
            if is_without_line(line, marker, comment)
                && without_line_of(line, marker, comment).is_none()
            {
                return false;
            }
            match fence_of(line, marker, comment) {
                Some(Fence::Begin) => open.push(marker),
                // the guard closes the innermost block open, whichever it was
                Some(Fence::End) if open.pop() != Some(marker) => return false,
                Some(Fence::End) | None => {}
            }
        }
    }
    open.is_empty()
}

/// Returns `text` without the blocks of `marker`, each from its `begin` line (`// marker:
/// begin`, or the same in the file's `comment`) to its `end` line, both included, and whether
/// any was dropped. Whatever is inside goes with it, blocks of its own marker or of another
/// included. Asked only of a text `are_markers_whole` passed: here a block left open would run
/// to the end.
pub(crate) fn remove_marked_blocks(text: &str, marker: &str, comment: Comment) -> (String, bool) {
    let mut depth = 0_usize;
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| match fence_of(line, marker, comment) {
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

/// Returns `text` without the lines that end with `marker` as a comment of the file's kind
/// (`// marker`, `# marker`, `<!-- marker -->`), and whether any was dropped. The comment is
/// matched whole, so `camera: first-person` never takes a line marked `camera: first`.
pub(crate) fn remove_marked_lines(text: &str, marker: &str, comment: Comment) -> (String, bool) {
    let mark = comment.around(marker);
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim_end().ends_with(&mark))
        .collect();
    let dropped = kept.len() != text.lines().count();
    let mut new_text = kept.join("\n");
    if text.ends_with('\n') && !new_text.is_empty() {
        new_text.push('\n');
    }
    (new_text, dropped)
}

/// Returns `text` with every without line of `marker` (`// without marker: <line>`, or the same
/// in the file's `comment`) turned into the line it holds, at the comment's indentation, and
/// whether any was. A game that leaves the option out gets the line that stands in for what
/// went.
pub(crate) fn restore_without_lines(text: &str, marker: &str, comment: Comment) -> (String, bool) {
    let mut restored = false;
    let lines: Vec<String> = text
        .lines()
        .map(|line| match without_line_of(line, marker, comment) {
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
pub(crate) fn remove_without_lines(text: &str, marker: &str, comment: Comment) -> (String, bool) {
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| !is_without_line(line, marker, comment))
        .collect();
    let dropped = kept.len() != text.lines().count();
    let mut new_text = kept.join("\n");
    if text.ends_with('\n') && !new_text.is_empty() {
        new_text.push('\n');
    }
    (new_text, dropped)
}

/// Returns Markdown `text` without the markers of `marker`, an option the game keeps, and
/// whether any went: a line marker is cut off the end of its line, which stays, and a block's
/// fences are dropped, what is between them staying. In the docs a marker serves nothing, and
/// inside a code block it would show as text in the game's own docs; in code it stays, showing
/// what belongs to the option.
pub(crate) fn remove_markdown_markers(text: &str, marker: &str) -> (String, bool) {
    let mark = Comment::Html.around(marker);
    let mut changed = false;
    let lines: Vec<&str> = text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_end();
            if fence_of(line, marker, Comment::Html).is_some() || trimmed.trim_start() == mark {
                changed = true;
                None
            } else if let Some(rest) = trimmed.strip_suffix(&mark) {
                changed = true;
                Some(rest.trim_end())
            } else {
                Some(line)
            }
        })
        .collect();
    let mut new_text = lines.join("\n");
    if text.ends_with('\n') && !new_text.is_empty() {
        new_text.push('\n');
    }
    (new_text, changed)
}

/// Tells whether `line` starts as a without line of `marker`, `// without marker: `, whole or
/// not: whether it ends as one is `without_line_of`'s to say.
fn is_without_line(line: &str, marker: &str, comment: Comment) -> bool {
    let opening = format!("{}without {marker}: ", comment.open());
    line.trim_start().starts_with(&opening)
}

/// Returns the line a without line of `marker` holds, at the comment's indentation, if `line`
/// is one, closed once at its end. Matched whole: the marker must be followed by `: `, so
/// `camera: first` never takes `// without camera: first-person: …`, which is why no marker
/// and `: ` may start another.
fn without_line_of(line: &str, marker: &str, comment: Comment) -> Option<String> {
    let body = line.trim_start();
    let indent = &line[..line.len() - body.len()];
    let opening = format!("{}without {marker}: ", comment.open());
    let code = body
        .strip_prefix(&opening)?
        .trim_end()
        .strip_suffix(comment.close())?;
    if comment == Comment::Html && code.contains("-->") {
        return None;
    }
    Some(format!("{indent}{code}"))
}

/// Returns which fence of `marker`'s blocks `line` is, if it is one: the whole line, spaces
/// aside, is `marker: begin` or `marker: end` as a comment of the file's kind. Matched whole,
/// so `camera: first` never opens a block of `camera: first-person`.
fn fence_of(line: &str, marker: &str, comment: Comment) -> Option<Fence> {
    let line = line.trim();
    if line == comment.around(&format!("{marker}: begin")) {
        Some(Fence::Begin)
    } else if line == comment.around(&format!("{marker}: end")) {
        Some(Fence::End)
    } else {
        None
    }
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_inside_another_is_whole() {
        let text = "// multiplayer: begin\n// split screen: begin\nx\n// split screen: end\n\
                    // multiplayer: end\n";
        assert!(are_markers_whole(
            text,
            &["multiplayer", "split screen"],
            Comment::Slashes
        ));
    }

    #[test]
    fn a_block_never_closed_is_broken() {
        let text = "a\n// multiplayer: begin\nx\n";
        assert!(!are_markers_whole(text, &["multiplayer"], Comment::Slashes));
    }

    #[test]
    fn an_end_with_nothing_to_close_is_broken() {
        let text = "x\n// multiplayer: end\na\n";
        assert!(!are_markers_whole(text, &["multiplayer"], Comment::Slashes));
    }

    #[test]
    fn blocks_that_cross_are_broken() {
        let text = "// multiplayer: begin\n// split screen: begin\n// multiplayer: end\n\
                    // split screen: end\n";
        assert!(!are_markers_whole(
            text,
            &["multiplayer", "split screen"],
            Comment::Slashes
        ));
    }

    #[test]
    fn a_marked_block_goes_with_its_fences() {
        let text = "a\n    // split screen: begin\n    x\n    // split screen: end\nb\n";
        assert_eq!(
            remove_marked_blocks(text, "split screen", Comment::Slashes),
            ("a\nb\n".to_string(), true)
        );
    }

    #[test]
    fn a_toml_block_goes_like_a_rust_one() {
        let text =
            "[dependencies]\n# multiplayer: begin\nlightyear = \"0.31\"\n# multiplayer: end\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer", Comment::Hash),
            ("[dependencies]\n".to_string(), true)
        );
    }

    #[test]
    fn a_block_inside_one_of_its_own_marker_goes_with_it() {
        let text = "// multiplayer: begin\na\n// multiplayer: begin\nb\n// multiplayer: end\nc\n\
                    // multiplayer: end\nd\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer", Comment::Slashes),
            ("d\n".to_string(), true)
        );
    }

    #[test]
    fn a_block_takes_the_blocks_of_other_markers_inside_it() {
        let text = "// multiplayer: begin\n// split screen: begin\nx\n// split screen: end\n\
                    // multiplayer: end\nd\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer", Comment::Slashes),
            ("d\n".to_string(), true)
        );
    }

    #[test]
    fn a_block_of_another_marker_stays() {
        let text = "// split screen: begin\nx\n// split screen: end\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer", Comment::Slashes),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_fence_is_matched_whole() {
        let text = "// camera: first-person: begin\nx\n// camera: first-person: end\n";
        assert_eq!(
            remove_marked_blocks(text, "camera: first", Comment::Slashes),
            (text.to_string(), false)
        );
    }

    #[test]
    fn marked_lines_go_and_the_rest_stays() {
        let text = "mod a; // multiplayer\nmod b;\n";
        assert_eq!(
            remove_marked_lines(text, "multiplayer", Comment::Slashes),
            ("mod b;\n".to_string(), true)
        );
    }

    #[test]
    fn a_toml_marked_line_goes() {
        let text = "[dependencies]\nlightyear = \"0.31\" # multiplayer\n";
        assert_eq!(
            remove_marked_lines(text, "multiplayer", Comment::Hash),
            ("[dependencies]\n".to_string(), true)
        );
    }

    #[test]
    fn a_rust_file_reads_no_hash_marker() {
        let text = "let a = 1; # multiplayer\n";
        assert_eq!(
            remove_marked_lines(text, "multiplayer", Comment::Slashes),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_marker_is_matched_whole() {
        let text = "mod first_person; // camera: first-person\n";
        assert_eq!(
            remove_marked_lines(text, "camera: first", Comment::Slashes),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_without_line_becomes_its_line_at_its_indentation() {
        let text = "struct A {\n    // without split screen: pads: Query<&Gamepad>,\n}\n";
        assert_eq!(
            restore_without_lines(text, "split screen", Comment::Slashes),
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
            restore_without_lines(text, "multiplayer", Comment::Hash),
            ("name = \"solo\"\n".to_string(), true)
        );
    }

    #[test]
    fn a_without_line_of_another_marker_stays_a_comment() {
        let text = "// without multiplayer: mod files;\n";
        assert_eq!(
            restore_without_lines(text, "split screen", Comment::Slashes),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_without_line_is_matched_whole() {
        let text = "// without camera: first-person: mod a;\n";
        assert_eq!(
            restore_without_lines(text, "camera: first", Comment::Slashes),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_kept_option_loses_its_without_lines() {
        let text = "a\n    // without split screen: b\nc\n";
        assert_eq!(
            remove_without_lines(text, "split screen", Comment::Slashes),
            ("a\nc\n".to_string(), true)
        );
    }

    // ---- markdown and fluent ------------------------------------------------

    #[test]
    fn markdown_takes_html_comments() {
        assert_eq!(comment_for("md"), Some(Comment::Html));
    }

    #[test]
    fn a_locale_takes_hash_comments() {
        assert_eq!(comment_for("ftl"), Some(Comment::Hash));
    }

    #[test]
    fn a_file_of_another_kind_has_no_markers() {
        assert_eq!(comment_for("png"), None);
    }

    #[test]
    fn a_markdown_heading_is_no_marker() {
        let text = "# multiplayer\ntext\n";
        assert_eq!(
            remove_marked_lines(text, "multiplayer", Comment::Html),
            (text.to_string(), false)
        );
    }

    #[test]
    fn a_markdown_line_marker_takes_its_line() {
        let text = "a\nb <!-- multiplayer -->\n| c | d | <!-- multiplayer -->\ne\n";
        assert_eq!(
            remove_marked_lines(text, "multiplayer", Comment::Html),
            ("a\ne\n".to_string(), true)
        );
    }

    #[test]
    fn a_markdown_block_goes_with_its_fences() {
        let text = "<!-- multiplayer: begin -->\n- item\n<!-- multiplayer: end -->\nnext\n";
        assert_eq!(
            remove_marked_blocks(text, "multiplayer", Comment::Html),
            ("next\n".to_string(), true)
        );
    }

    #[test]
    fn a_markdown_without_line_becomes_its_line() {
        let text = "    <!-- without camera: third-person: physics_debug.rs  F6 -->\n";
        assert_eq!(
            restore_without_lines(text, "camera: third-person", Comment::Html),
            ("    physics_debug.rs  F6\n".to_string(), true)
        );
    }

    #[test]
    fn a_markdown_without_line_holding_its_close_is_broken() {
        let text = "<!-- without multiplayer: a --> b -->\n";
        assert!(!are_markers_whole(text, &["multiplayer"], Comment::Html));
    }

    #[test]
    fn a_markdown_without_line_never_closed_is_broken() {
        let text = "<!-- without multiplayer: a\nb\n";
        assert!(!are_markers_whole(text, &["multiplayer"], Comment::Html));
    }

    #[test]
    fn a_kept_option_leaves_no_marker_in_markdown() {
        let text = "a <!-- multiplayer -->\n<!-- multiplayer: begin -->\nb\n\
                    <!-- multiplayer: end -->\n<!-- multiplayer -->\nc\n";
        assert_eq!(
            remove_markdown_markers(text, "multiplayer"),
            ("a\nb\nc\n".to_string(), true)
        );
    }

    #[test]
    fn a_kept_option_leaves_other_markers_in_markdown() {
        let text = "a <!-- split screen -->\n";
        assert_eq!(
            remove_markdown_markers(text, "multiplayer"),
            (text.to_string(), false)
        );
    }
}
