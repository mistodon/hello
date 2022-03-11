pub mod git;

use std::path::*;

use color_eyre::Result;
use owo_colors::OwoColorize;
use structopt::StructOpt;

use crate::git::{GitReporter, RepositoryState};

#[derive(StructOpt)]
pub struct HelloArgs {
    pub path: Option<PathBuf>,
}

pub fn report(args: &HelloArgs) -> Result<()> {
    let path: Option<&Path> = args.path.as_ref().map(|p| p.as_ref());
    let path = path.unwrap_or(".".as_ref());

    let mut git = GitReporter::new(path)?;

    println!(
        "{}",
        path.canonicalize()?.display().bright_yellow().underline()
    );
    for remote in git.remotes()? {
        println!("{} -> {}", remote.name.green(), remote.url.cyan());
    }
    print!("{} = ", "user".green());
    match git.config()?.user {
        Some(user) => println!("{} <{}>", user.name.cyan(), user.email.cyan()),
        None => println!("{}", "None".red()),
    }

    let staged_files = git.staged_files()?;
    let change_files = git.change_files()?;
    let frozen_files = git.frozen_files()?;

    let branch = git.current_branch().ok();
    match branch {
        Some(branch) => print!("[{}] (", branch.bright_cyan()),
        None => print!("[{}] (", "unknown branch".magenta()),
    }

    {
        use RepositoryState::*;

        let state = git.state();
        let state_label = format!("{:?}", state).to_lowercase();
        match state {
            Clean => print!("{}", state_label.bright_green()),
            _ => print!("{}", state_label.bright_red()),
        }
    }

    if staged_files.len() > 0 {
        print!(" | {}", "staged".green());
    }
    if change_files.len() > 0 {
        print!(" | {}", "changed".magenta());
    }
    println!(")");

    print!("{} {}  ", staged_files.len().green(), "staged".green());
    print!("{} {}  ", change_files.len().magenta(), "changes".magenta());
    print!("{} {}  ", frozen_files.len().cyan(), "frozen".cyan());
    print!("{} {}  ", git.stash_len()?.yellow(), "stashed".yellow());
    println!();

    for frozen_file in &frozen_files {
        println!("{}", frozen_file.cyan());
    }

    Ok(())
}
