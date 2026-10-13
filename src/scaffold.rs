use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fmt, fs};

use crate::choices::{Choices, branch_for, project_path, remove_section, replace_line};

// ==== Config =================================================================

/// Where every game is cloned from.
const TEMPLATE_URL: &str = "https://github.com/Heryk13/bevy-starter.git";

/// The message of the game's first commit, in the template's Conventional Commits style.
const FIRST_COMMIT_MESSAGE: &str = "build: start from bevy-starter";

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
    edit_file(&path.join("README.md"), |text| {
        remove_section(text, "## Branches")
    })?;

    // ---- a history of its own -----------------------------------------------
    run_git(path, &["init"])?;
    run_git(path, &["add", "--all"])?;
    run_git(path, &["commit", "--message", FIRST_COMMIT_MESSAGE])?;
    Ok(())
}

/// Rewrites `file` on disk with `edit`, one of the pure edits in `choices.rs`. When `edit` finds
/// nothing to change, the template moved what the CLI edits, and that is reported instead of
/// leaving the game half made.
fn edit_file(file: &Path, edit: impl FnOnce(&str) -> Option<String>) -> Result<(), ScaffoldError> {
    let text =
        fs::read_to_string(file).map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
    let new_text = edit(&text).ok_or_else(|| ScaffoldError::TemplateChanged(file.to_path_buf()))?;
    fs::write(file, new_text).map_err(|error| ScaffoldError::EditFailed(error.to_string()))?;
    Ok(())
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
