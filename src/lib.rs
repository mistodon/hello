pub mod git;

use std::path::*;

use color_eyre::Result;
use owo_colors::OwoColorize;
use structopt::{clap::Shell, StructOpt};

use crate::git::{GitReporter, RepositoryState};

#[derive(StructOpt)]
pub struct HelloArgs {
    #[structopt(short = "p", long = "path")]
    pub path: Option<PathBuf>,

    #[structopt(subcommand)]
    pub sub: Option<HelloSubCmd>,
}

#[derive(StructOpt)]
pub enum HelloSubCmd {
    Completion { shell: Shell },
    Freeze { paths: Vec<PathBuf> },
    Unfreeze { paths: Vec<PathBuf> },
}

pub fn report(args: &HelloArgs) -> Result<()> {
    let path: Option<&Path> = args.path.as_ref().map(|p| p.as_ref());
    let path = path.unwrap_or(".".as_ref());

    let mut git = GitReporter::new(path)?;

    match &args.sub {
        None => {
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
        }
        Some(HelloSubCmd::Freeze { paths }) => {
            validate_paths(paths)?;
            git.set_files_frozen(paths, true)?;
        }
        Some(HelloSubCmd::Unfreeze { paths }) => {
            validate_paths(paths)?;
            git.set_files_frozen(paths, false)?;
        }
        Some(HelloSubCmd::Completion { shell }) => {
            let mut app = HelloArgs::clap();
            let mut buffer = vec![];
            let mut buffer = std::io::Cursor::new(&mut buffer);
            app.gen_completions_to("hi", *shell, &mut buffer);

            println!("{}", std::str::from_utf8(buffer.into_inner())?);
        }
    }

    Ok(())
}

fn validate_paths<P: AsRef<Path>>(paths: &[P]) -> Result<()> {
    paths
        .iter()
        .all(|p| p.as_ref().exists())
        .then(|| ())
        .ok_or_else(|| color_eyre::eyre::eyre!("One or more paths provided do not exist"))
}
