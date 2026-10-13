use inquire::{InquireError, error::InquireResult};
use std::env;
use std::path::PathBuf;
use std::process;

use crate::choices::{Choices, is_valid_name};

mod choices;
mod prompts;
mod scaffold;

fn main() -> InquireResult<()> {
    if !scaffold::is_git_installed() {
        eprintln!("Git is not installed. Please install Git to continue.");
        process::exit(1);
    }
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
        Ok(Some(choices)) => {
            let result = scaffold::create_game(&choices);
            if let Err(error) = result {
                eprintln!("{}", error);
                process::exit(1);
            }
        }
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
