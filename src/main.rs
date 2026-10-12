use inquire::{InquireError, error::InquireResult};
use std::env;
use std::path::PathBuf;
use std::process;

use crate::choices::{Choices, is_valid_name};

#[expect(
    dead_code,
    reason = "the branch and the file edits wait for `scaffold.rs` (spec step 6)"
)]
mod choices;
mod prompts;

fn main() -> InquireResult<()> {
    let mut args = env::args().skip(1);
    let name = args.next();
    let folder = args.next().map(PathBuf::from);
    if let Some(name) = &name
        && !is_valid_name(name)
    {
        // no question to ask again here, so the rule is said once and the CLI stops
        eprintln!(
            "`{name}` is not a valid project name. {}",
            prompts::NAME_RULE
        );
        process::exit(1);
    }

    match ask(name, folder) {
        // `scaffold.rs` builds the game from these choices here (spec step 6)
        Ok(Some(_choices)) => {}
        // Esc is read as Ctrl+C: both mean the user wants out, and nothing was written yet
        Ok(None) | Err(InquireError::OperationInterrupted | InquireError::OperationCanceled) => {
            println!("Nothing was created.");
        }
        Err(error) => return Err(error),
    }
    Ok(())
}

/// Asks the questions the arguments left open, then the confirmation. Gives the choices back
/// only when the user said yes.
fn ask(name: Option<String>, folder: Option<PathBuf>) -> InquireResult<Option<Choices>> {
    let choices = prompts::ask_choices(name, folder)?;
    Ok(prompts::ask_confirmation(&choices)?.then_some(choices))
}
