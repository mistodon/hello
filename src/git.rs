use std::path::Path;

use color_eyre::Result;
use git2::{IndexEntry, Repository, Status};

pub use git2::RepositoryState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub user: Option<User>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

pub struct GitReporter {
    repo: Repository,
}

impl GitReporter {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path.as_ref())?;
        Ok(GitReporter { repo })
    }

    pub fn config(&self) -> Result<Config> {
        let config = self.repo.config()?;

        let get_user = || {
            let name = config.get_string("user.name").ok()?;
            let email = config.get_string("user.email").ok()?;
            Some(User { name, email })
        };

        let user = get_user();

        Ok(Config { user })
    }

    pub fn remotes(&self) -> Result<Vec<Remote>> {
        let mut remotes = vec![];
        for name in self
            .repo
            .remotes()?
            .iter_bytes()
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        {
            let remote = self.repo.find_remote(&name)?;
            remotes.push(Remote {
                name,
                url: String::from_utf8_lossy(remote.url_bytes()).into_owned(),
            });
        }
        Ok(remotes)
    }

    const FREEZE_FLAG: u16 = 1 << 15;

    fn files_where<P: Fn(&IndexEntry) -> bool>(&self, p: P) -> Result<Vec<IndexEntry>> {
        Ok(self.repo.index()?.iter().filter(p).collect())
    }

    pub fn frozen_files(&self) -> Result<Vec<String>> {
        Ok(self
            .files_where(|entry| entry.flags & Self::FREEZE_FLAG != 0)?
            .into_iter()
            .map(|entry| String::from_utf8_lossy(&entry.path).into_owned())
            .collect())
    }

    pub fn set_files_frozen<P: AsRef<Path>>(&mut self, paths: &[P], frozen: bool) -> Result<()> {
        let mut index = self.repo.index()?;

        for path in paths {
            let mut entry = index.get_path(path.as_ref(), 0).unwrap();
            if frozen {
                entry.flags |= Self::FREEZE_FLAG;
            } else {
                entry.flags &= !Self::FREEZE_FLAG;
            }
            index.add(&entry)?;
        }

        self.repo.set_index(&mut index)?;
        index.write()?;

        Ok(())
    }

    pub fn staged_files(&self) -> Result<Vec<String>> {
        let modified_flags = Status::INDEX_NEW
            | Status::INDEX_MODIFIED
            | Status::INDEX_DELETED
            | Status::INDEX_TYPECHANGE
            | Status::INDEX_RENAMED;
        Ok(self
            .repo
            .statuses(None)?
            .iter()
            .filter(|entry| entry.status().intersects(modified_flags))
            .map(|entry| String::from_utf8_lossy(entry.path_bytes()).into_owned())
            .collect())
    }

    pub fn change_files(&self) -> Result<Vec<String>> {
        let modified_flags = Status::WT_NEW
            | Status::WT_MODIFIED
            | Status::WT_DELETED
            | Status::WT_TYPECHANGE
            | Status::WT_RENAMED;
        Ok(self
            .repo
            .statuses(None)?
            .iter()
            .filter(|entry| entry.status().intersects(modified_flags))
            .map(|entry| String::from_utf8_lossy(entry.path_bytes()).into_owned())
            .collect())
    }

    pub fn stash_len(&mut self) -> Result<usize> {
        let mut count = 0;
        self.repo.stash_foreach(|_, _, _| {
            count += 1;
            true
        })?;
        Ok(count)
    }

    pub fn state(&self) -> RepositoryState {
        self.repo.state()
    }

    fn head_branch(&self) -> Result<String> {
        let branch = self
            .repo
            .branches(None)?
            .filter_map(|branch| {
                branch
                    .ok()
                    .and_then(|branch| branch.0.is_head().then(|| branch.0))
            })
            .next()
            .ok_or_else(|| color_eyre::eyre::eyre!("No HEAD branch found"))?;
        Ok(String::from_utf8_lossy(branch.name_bytes()?).into_owned())
    }

    fn head_ref(&self) -> Result<String> {
        Ok(self
            .repo
            .find_reference("HEAD")?
            .symbolic_target_bytes()
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .ok_or_else(|| color_eyre::eyre::eyre!("No HEAD branch found"))?)
    }

    pub fn current_branch(&self) -> Result<String> {
        self.head_branch().or(self.head_ref())
    }
}
