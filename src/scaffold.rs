use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fmt, fs};

use crate::choices::{Choices, Fate, OPTIONS, Removal, branch_for, fate_of, project_path};
use crate::edits::{
    are_blocks_whole, remove_marked_blocks, remove_marked_lines, remove_section,
    remove_without_lines, replace_line, restore_without_lines,
};

// ==== Config =================================================================

/// Where every game is cloned from.
const TEMPLATE_URL: &str = "https://github.com/Heryk13/bevy-starter.git";

/// The message of the game's first commit, in the template's Conventional Commits style.
const FIRST_COMMIT_MESSAGE: &str = "build: start from bevy-starter";

/// The heading of the section about the template's branch tree, the same in `README.md` and in
/// `CONTRIBUTING.md`.
const BRANCHES_HEADING: &str = "## Branches";

// ==== Types ==================================================================

/// Why a game could not be created. Each variant is a failure the CLI can meet and tell apart.
pub(crate) enum ScaffoldError {
    FolderNotEmpty(PathBuf),
    CloneFailed(String),
    RemoveFailed(String),
    TemplateChanged(PathBuf),
    EditFailed(String),
    GitFailed(String),
}

/// Gives the message the user reads, which says what went wrong and what to do next.
impl fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaffoldError::FolderNotEmpty(path) => {
                write!(
                    f,
                    "{} already exists and is not empty. Choose another name or an empty folder.",
                    path.display()
                )
            }
            ScaffoldError::CloneFailed(message) => {
                write!(
                    f,
                    "Failed to clone the template: {}. Check your internet connection, or wait a \
                     moment, and try again.",
                    message
                )
            }
            ScaffoldError::RemoveFailed(message) => {
                write!(
                    f,
                    "Could not remove a file the template does not need: {}. Close any program \
                     using the folder, such as an editor or the file explorer, and try again.",
                    message
                )
            }
            ScaffoldError::TemplateChanged(path) => {
                write!(
                    f,
                    "{} is not what this create-bevy-starter expects: the template changed and the \
                     CLI did not follow. Update create-bevy-starter, or report it to its author.",
                    path.display()
                )
            }
            ScaffoldError::EditFailed(message) => {
                write!(
                    f,
                    "Could not edit a file of the new game: {}. Close any program using the \
                     folder, such as an editor or the file explorer, and try again.",
                    message
                )
            }
            ScaffoldError::GitFailed(message) => {
                // a first commit most often fails on a git that was never told who commits
                write!(
                    f,
                    "Could not start the game's git history: {}. If git asks who you are, run \
                     `git config --global user.name \"Your Name\"` and `git config --global \
                     user.email you@example.com`, then try again.",
                    message
                )
            }
        }
    }
}

// ==== Checks =================================================================

/// Tells whether `git` runs on this machine, since the template is cloned with it.
pub(crate) fn is_git_installed() -> bool {
    let output = Command::new("git").arg("--version").output();
    if let Ok(output) = output
        && output.status.success()
    {
        return true;
    }
    false
}

/// Tells whether the game can be created at `path`: nothing there yet, or an empty folder.
/// Anything else, a file or a folder that cannot be read included, is refused, so nothing is
/// ever overwritten.
fn is_folder_free(path: &Path) -> bool {
    !path.exists() || fs::read_dir(path).is_ok_and(|mut entries| entries.next().is_none())
}

// ==== Creating ===============================================================

/// Builds the game the user chose, in `<folder>/<name>`. Stops before writing anything when the
/// folder is taken.
pub(crate) fn create_game(choices: &Choices) -> Result<(), ScaffoldError> {
    let path = project_path(&choices.folder, &choices.name);
    if !is_folder_free(&path) {
        return Err(ScaffoldError::FolderNotEmpty(path));
    }
    if let Err(error) = build_game(&path, choices) {
        // a half-built game is worse than none; a failed cleanup is dropped, since the error
        // the user needs is the one that stopped the build
        let _ = fs::remove_dir_all(&path);
        return Err(error);
    }
    Ok(())
}

/// Runs every step that writes the game to disk. Any failure here leaves a half-built folder,
/// which `create_game` deletes.
fn build_game(path: &Path, choices: &Choices) -> Result<(), ScaffoldError> {
    // ---- the clone ----------------------------------------------------------
    let branch = branch_for(&choices.kind);
    let output = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", branch, TEMPLATE_URL])
        .arg(path)
        .output()
        .map_err(|error| ScaffoldError::CloneFailed(error.to_string()))?;
    if !output.status.success() {
        return Err(ScaffoldError::CloneFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    // the game starts its own history: the template's would show in its log, and its push would
    // reach the template's repository
    let git_dir = path.join(".git");
    fs::remove_dir_all(git_dir).map_err(|error| ScaffoldError::RemoveFailed(error.to_string()))?;

    // ---- the three names ----------------------------------------------------
    edit_file(&path.join("Cargo.toml"), |text| {
        replace_line(text, "name = ", &format!("name = \"{}\"", choices.name))
    })?;
    edit_file(&path.join("justfile"), |text| {
        replace_line(text, "game := ", &format!("game := \"{}\"", choices.name))
    })?;
    edit_file(&path.join("src/main.rs"), |text| {
        replace_line(
            text,
            "pub(crate) const GAME_TITLE",
            &format!("pub(crate) const GAME_TITLE: &str = \"{}\";", choices.title),
        )
    })?;

    // ---- what only the template needs ---------------------------------------
    // the specs are the template's history; the folder stays for the game's own, kept by a
    // `.gitkeep` since git drops empty folders
    let specs_dir = path.join("docs/specs");
    fs::remove_dir_all(&specs_dir)
        .map_err(|error| ScaffoldError::RemoveFailed(error.to_string()))?;
    fs::create_dir(&specs_dir).map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
    fs::write(specs_dir.join(".gitkeep"), "")
        .map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
    // a game has no branch tree: the README's section lists the template's branches, and
    // CONTRIBUTING's holds their rules and the tools that keep them in step
    for file in ["README.md", "CONTRIBUTING.md"] {
        edit_file(&path.join(file), |text| {
            remove_section(text, BRANCHES_HEADING)
        })?;
    }
    remove_tools(path)?;

    // ---- what the game was not given ----------------------------------------
    check_blocks(path)?;
    let mut changed = false;
    for option in &OPTIONS {
        changed |= match fate_of(choices, option) {
            Fate::Kept => keep_option(path, option)?,
            Fate::LeftOut => remove_left_out(path, option, true)?,
            Fate::NotOffered => remove_left_out(path, option, false)?,
        };
    }
    if changed {
        tidy_after_removals(path);
    }

    // ---- a history of its own -----------------------------------------------
    run_git(path, &["init"])?;
    run_git(path, &["add", "--all"])?;
    run_git(path, &["commit", "--message", FIRST_COMMIT_MESSAGE])?;
    Ok(())
}

/// Rewrites `file` on disk with `edit`, one of the pure edits in `edits.rs`. When `edit` finds
/// nothing to change, the template moved what the CLI edits, and that is reported instead of
/// leaving the game half made.
fn edit_file(file: &Path, edit: impl FnOnce(&str) -> Option<String>) -> Result<(), ScaffoldError> {
    let text =
        fs::read_to_string(file).map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
    let new_text = edit(&text).ok_or_else(|| ScaffoldError::TemplateChanged(file.to_path_buf()))?;
    fs::write(file, new_text).map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
    Ok(())
}

/// Removes the template's `tools/` from the game at `path`: they commit and merge down the
/// template's branch tree, which a game does not have. A template without the folder is left
/// as it is, since none of its branches had one before the tools were written.
fn remove_tools(path: &Path) -> Result<(), ScaffoldError> {
    let tools_dir = path.join("tools");
    if !tools_dir.exists() {
        return Ok(());
    }
    fs::remove_dir_all(tools_dir).map_err(|error| ScaffoldError::RemoveFailed(error.to_string()))
}

/// Refuses a game whose template has a broken block in a file markers are read in: one never
/// closed, an `end` with nothing to close, or two that cross. Asked once, before anything is
/// removed, for every option, kept or not: a broken block is the template's fault whatever was
/// chosen, and removing it would eat the rest of its file.
fn check_blocks(path: &Path) -> Result<(), ScaffoldError> {
    let markers = OPTIONS.map(|option| option.marker);
    for file in marked_files(path)? {
        let text = fs::read_to_string(&file)
            .map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
        if !are_blocks_whole(&text, &markers) {
            return Err(ScaffoldError::TemplateChanged(file));
        }
    }
    Ok(())
}

/// Removes from the game at `path` what it was not given: `removal`'s files, and every block
/// and line carrying its marker in `Cargo.toml` and the Rust sources, and tells whether
/// anything went. When the kind `offered` the option, none of its files being there means the
/// template moved them, and the game would keep what was left out; when it did not, the files
/// are not expected, and only the markers are read.
fn remove_left_out(path: &Path, removal: &Removal, offered: bool) -> Result<bool, ScaffoldError> {
    let mut found = false;
    for file in removal.files {
        let file = path.join(file);
        let removed = if file.is_dir() {
            fs::remove_dir_all(&file)
        } else if file.is_file() {
            fs::remove_file(&file)
        } else {
            continue;
        };
        removed.map_err(|error| ScaffoldError::RemoveFailed(error.to_string()))?;
        found = true;
    }
    if offered && !found {
        let first = removal.files.first().copied().unwrap_or_default();
        return Err(ScaffoldError::TemplateChanged(path.join(first)));
    }
    let mut changed = found;
    for source in marked_files(path)? {
        let text = fs::read_to_string(&source)
            .map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
        let (without_blocks, dropped_blocks) = remove_marked_blocks(&text, removal.marker);
        let (without_lines, dropped_lines) = remove_marked_lines(&without_blocks, removal.marker);
        let (new_text, restored) = restore_without_lines(&without_lines, removal.marker);
        if dropped_blocks || dropped_lines || restored {
            fs::write(&source, new_text)
                .map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
            changed = true;
        }
    }
    Ok(changed)
}

/// Leaves `removal`'s option in the game at `path`, tidied: only its without lines go,
/// commented-out code a game that has the option will never use. Its line markers and blocks
/// stay, showing what belongs to it. Tells whether any file changed.
fn keep_option(path: &Path, removal: &Removal) -> Result<bool, ScaffoldError> {
    let mut changed = false;
    for source in marked_files(path)? {
        let text = fs::read_to_string(&source)
            .map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
        let (new_text, dropped) = remove_without_lines(&text, removal.marker);
        if dropped {
            fs::write(&source, new_text)
                .map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
            changed = true;
        }
    }
    Ok(changed)
}

/// Returns the files of the game at `path` that markers are read in: `Cargo.toml` and every
/// Rust source.
fn marked_files(path: &Path) -> Result<Vec<PathBuf>, ScaffoldError> {
    let mut files = vec![path.join("Cargo.toml")];
    collect_rust_files(&path.join("src"), &mut files)?;
    Ok(files)
}

/// Adds every `.rs` file under `dir`, at any depth, to `files`.
fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), ScaffoldError> {
    let entries =
        fs::read_dir(dir).map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_rust_files(&entry_path, files)?;
        } else if entry_path
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            files.push(entry_path);
        }
    }
    Ok(())
}

/// Leaves the game as its own `just check` wants it once lines are gone: formatted again, since
/// a call whose marked argument went can now fit one line, or end without its comma; and its
/// `Cargo.lock` without the crates only what was left out needed, which asking cargo for the
/// game's metadata drops, updating nothing else. Best effort: the game builds either way, and a
/// cargo that cannot run here is no reason to fail.
fn tidy_after_removals(path: &Path) {
    for args in [&["fmt"][..], &["metadata", "--format-version", "1"]] {
        let _ = Command::new("cargo").args(args).current_dir(path).output();
    }
}

/// Runs `git` with `args` inside the game's folder, and gives back what git said when it fails.
fn run_git(path: &Path, args: &[&str]) -> Result<(), ScaffoldError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .map_err(|error| ScaffoldError::GitFailed(error.to_string()))?;
    if !output.status.success() {
        return Err(ScaffoldError::GitFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(())
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns an empty folder of the test's own under the system's temporary one, so tests
    /// running in parallel share no files and an earlier run leaves none behind.
    fn fresh_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        // a folder that is not there yet is the usual case, and nothing to report
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the test folder can be created");
        dir
    }

    #[test]
    fn a_game_keeps_no_tools_folder() {
        let game = fresh_test_dir("create-bevy-starter-tools-removed");
        fs::create_dir(game.join("tools")).expect("the tools folder can be created");
        fs::write(game.join("tools/sync.sh"), "").expect("a tool can be written");
        assert!(remove_tools(&game).is_ok());
        assert!(!game.join("tools").exists());
    }

    #[test]
    fn a_template_without_tools_still_makes_a_game() {
        let game = fresh_test_dir("create-bevy-starter-no-tools");
        assert!(remove_tools(&game).is_ok());
        assert!(game.exists());
    }

    #[test]
    fn what_is_left_out_goes_with_its_lines() {
        let game = fresh_test_dir("create-bevy-starter-left-out");
        fs::create_dir_all(game.join("src/multiplayer")).expect("a folder can be created");
        fs::write(game.join("src/multiplayer.rs"), "").expect("a file can be written");
        fs::write(
            game.join("src/main.rs"),
            "mod multiplayer; // multiplayer\nmod ui;\n",
        )
        .expect("a file can be written");
        fs::write(
            game.join("Cargo.toml"),
            "[dependencies]\nlightyear = \"0.31\" # multiplayer\n",
        )
        .expect("a file can be written");
        assert!(remove_left_out(&game, &crate::choices::MULTIPLAYER, true).is_ok());
        assert!(!game.join("src/multiplayer.rs").exists());
        assert!(!game.join("src/multiplayer").exists());
        let main = fs::read_to_string(game.join("src/main.rs")).expect("main.rs is there");
        assert_eq!(main, "mod ui;\n");
        let manifest = fs::read_to_string(game.join("Cargo.toml")).expect("Cargo.toml is there");
        assert_eq!(manifest, "[dependencies]\n");
    }

    #[test]
    fn what_is_left_out_goes_with_its_blocks() {
        let game = fresh_test_dir("create-bevy-starter-left-out-blocks");
        fs::create_dir(game.join("src")).expect("a folder can be created");
        fs::write(game.join("src/multiplayer.rs"), "").expect("a file can be written");
        fs::write(
            game.join("src/main.rs"),
            "mod ui;\n// multiplayer: begin\nmod net;\n// multiplayer: end\n",
        )
        .expect("a file can be written");
        fs::write(game.join("Cargo.toml"), "[package]\n").expect("a file can be written");
        assert!(remove_left_out(&game, &crate::choices::MULTIPLAYER, true).is_ok());
        let main = fs::read_to_string(game.join("src/main.rs")).expect("main.rs is there");
        assert_eq!(main, "mod ui;\n");
    }

    #[test]
    fn a_template_with_a_broken_block_is_refused() {
        let game = fresh_test_dir("create-bevy-starter-broken-block");
        fs::create_dir(game.join("src")).expect("a folder can be created");
        fs::write(
            game.join("src/main.rs"),
            "// multiplayer: begin\nmod net;\n",
        )
        .expect("a file can be written");
        fs::write(game.join("Cargo.toml"), "[package]\n").expect("a file can be written");
        assert!(matches!(
            check_blocks(&game),
            Err(ScaffoldError::TemplateChanged(_))
        ));
    }

    #[test]
    fn a_removal_whose_files_moved_is_refused() {
        let game = fresh_test_dir("create-bevy-starter-moved");
        fs::create_dir(game.join("src")).expect("a folder can be created");
        assert!(matches!(
            remove_left_out(&game, &crate::choices::MULTIPLAYER, true),
            Err(ScaffoldError::TemplateChanged(_))
        ));
    }

    #[test]
    fn an_option_not_offered_loses_its_blocks_without_its_files() {
        let game = fresh_test_dir("create-bevy-starter-not-offered");
        fs::create_dir(game.join("src")).expect("a folder can be created");
        fs::write(
            game.join("src/input.rs"),
            "mod a;\n// split screen: begin\nstruct GuestPad;\n// split screen: end\n",
        )
        .expect("a file can be written");
        fs::write(game.join("Cargo.toml"), "[package]\n").expect("a file can be written");
        assert!(remove_left_out(&game, &crate::choices::SPLIT_SCREEN, false).is_ok());
        let input = fs::read_to_string(game.join("src/input.rs")).expect("input.rs is there");
        assert_eq!(input, "mod a;\n");
    }

    #[test]
    fn what_is_left_out_leaves_its_without_lines_as_code() {
        let game = fresh_test_dir("create-bevy-starter-without-restored");
        fs::create_dir(game.join("src")).expect("a folder can be created");
        fs::write(game.join("src/multiplayer.rs"), "").expect("a file can be written");
        fs::write(
            game.join("src/main.rs"),
            "pub(crate) mod files; // multiplayer\n// without multiplayer: mod files;\n",
        )
        .expect("a file can be written");
        fs::write(game.join("Cargo.toml"), "[package]\n").expect("a file can be written");
        assert!(remove_left_out(&game, &crate::choices::MULTIPLAYER, true).is_ok());
        let main = fs::read_to_string(game.join("src/main.rs")).expect("main.rs is there");
        assert_eq!(main, "mod files;\n");
    }

    #[test]
    fn a_kept_option_keeps_no_without_line() {
        let game = fresh_test_dir("create-bevy-starter-without-kept");
        fs::create_dir(game.join("src")).expect("a folder can be created");
        fs::write(
            game.join("src/main.rs"),
            "pub(crate) mod files; // multiplayer\n// without multiplayer: mod files;\n",
        )
        .expect("a file can be written");
        fs::write(game.join("Cargo.toml"), "[package]\n").expect("a file can be written");
        assert!(keep_option(&game, &crate::choices::MULTIPLAYER).is_ok());
        let main = fs::read_to_string(game.join("src/main.rs")).expect("main.rs is there");
        assert_eq!(main, "pub(crate) mod files; // multiplayer\n");
    }

    // ---- every combination: `just check-combos` -----------------------------

    /// Returns the choices of every game the CLI can make, each in a folder of its own under
    /// `folder`.
    fn every_combination(folder: &Path) -> Vec<Choices> {
        use crate::choices::{Camera, Kind, Player3d};
        let mut kinds = vec![
            Kind::Empty,
            Kind::ThreeD(None),
            Kind::TwoD(false),
            Kind::TwoD(true),
        ];
        let camera_sets = [
            vec![Camera::FirstPerson],
            vec![Camera::ThirdPerson],
            vec![Camera::FirstPerson, Camera::ThirdPerson],
        ];
        for cameras in camera_sets {
            for split_screen in [false, true] {
                kinds.push(Kind::ThreeD(Some(Player3d {
                    cameras: cameras.clone(),
                    split_screen,
                })));
            }
        }
        let mut every = Vec::new();
        for kind in kinds {
            for multiplayer in [false, true] {
                let name = format!("combo-{}", every.len() + 1);
                every.push(Choices {
                    title: name.clone(),
                    name,
                    folder: folder.to_path_buf(),
                    kind: kind.clone(),
                    multiplayer,
                });
            }
        }
        every
    }

    #[test]
    #[ignore = "clones the template and runs each game's `just check`, for an hour or so: \
                `just check-combos`"]
    fn every_combination_passes_the_games_own_check() {
        let folder = fresh_test_dir("create-bevy-starter-combos");
        // one for every game, so Bevy is built once, not once a game
        let target = std::env::temp_dir().join("create-bevy-starter-combos-target");
        let mut failed = Vec::new();
        for choices in every_combination(&folder) {
            let label = format!(
                "{}: {:?}, multiplayer {}",
                choices.name, choices.kind, choices.multiplayer
            );
            println!("==== {label}");
            if let Err(error) = create_game(&choices) {
                failed.push(format!("{label}: {error}"));
                continue;
            }
            let checked = Command::new("just")
                .arg("check")
                .current_dir(project_path(&choices.folder, &choices.name))
                .env("CARGO_TARGET_DIR", &target)
                .status();
            if !checked.is_ok_and(|status| status.success()) {
                failed.push(label);
            }
        }
        assert!(
            failed.is_empty(),
            "these games fail their check: {failed:#?}"
        );
    }
}
