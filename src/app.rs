use git2;
use std::{error::Error, fs, os::unix::fs::symlink};

use crate::{config::ConfigManager, sh::cmd};

pub struct App {
    config: ConfigManager,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let config = ConfigManager::new()?;
        Ok(Self { config })
    }

    pub fn get_local_repo_path(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.config.load()?.repo_path)
    }

    pub fn get_github_repo(&self) -> Result<String, Box<dyn Error>> {
        let account = self.config.load()?.github_account;

        Ok(format!("git@github.com:{account}/my-dot-files.git",))
    }

    fn is_repo_ready(&self) -> Result<bool, Box<dyn Error>> {
        let local_repo_path = &self.config.load()?.repo_path;
        let remote_repo = self.get_github_repo()?;

        // check if the repository already exists
        if !std::path::Path::new(&local_repo_path).exists() {
            return Ok(false);
        }

        // check if the repository is a git repository
        let repo = git2::Repository::open(&local_repo_path)?;
        if repo.is_bare() {
            return Err("The repository path is a bare repository".into());
        }

        // check if the repository has a remote and if it is the correct remote
        let remote = repo.find_remote("origin")?;
        if remote.url().ok_or("Unable to determine remote URL")? != remote_repo {
            return Err("The repository has a remote that is not the correct remote".into());
        }

        Ok(true)
    }

    pub fn init(&self, github_account: &str) -> Result<(), Box<dyn Error>> {
        self.config.init(github_account)
    }

    fn init_repo(&self) -> Result<(), Box<dyn Error>> {
        let local_repo_path = &self.config.load()?.repo_path;
        let remote_repo = self.get_github_repo()?;

        // check if the repository already exists
        if std::path::Path::new(&local_repo_path).exists() {
            // check if the repository is a git repository
            let repo = git2::Repository::open(&local_repo_path)?;
            if repo.is_bare() {
                return Err("The repository path is a bare repository".into());
            }

            // check if the repository has a remote and if it is the correct remote
            let remote = repo.find_remote("origin")?;
            if remote.url().ok_or("Unable to determine remote URL")? != remote_repo {
                return Err("The repository has a remote that is not the correct remote".into());
            }

            println!("The repository already exists at {local_repo_path}");
            return Ok(());
        }

        // create the parent directory for the repository
        let repo_parent_dir = std::path::Path::new(&local_repo_path)
            .parent()
            .ok_or("Unable to determine repository directory")?;
        if !repo_parent_dir.exists() {
            fs::create_dir_all(repo_parent_dir)?;
        }

        println!("Cloning the repository from {remote_repo} to {local_repo_path}");
        cmd!("git clone {remote_repo} {local_repo_path}")?;
        Ok(())
    }

    pub fn add_dotfile(&self, dotfile: &str) -> Result<(), Box<dyn Error>> {
        self.config.add_dotfile(dotfile)?;

        // link the dotfile
        let home = env!("HOME");
        let dotfile_path = format!("{home}/{dotfile}");
        let repo_path = &self.config.load()?.repo_path;
        let repo_dotfile_path = format!("{repo_path}/{dotfile}");

        // move the dotfile to the repository
        fs::rename(&dotfile_path, &repo_dotfile_path)?;

        // create a symlink to the dotfile
        symlink(repo_dotfile_path, dotfile_path)?;

        // add the dotfile to the repository
        cmd!("git -C {repo_path} add {dotfile}")?;

        Ok(())
    }

    pub fn is_clean(&self) -> Result<bool, Box<dyn Error>> {
        if !self.is_repo_ready()? {
            println!("The repository is not ready");
            return Ok(true);
        }

        let repo_path = &self.config.load()?.repo_path;
        let stdout = cmd!("git -C {repo_path} status --porcelain")?;

        Ok(stdout.is_empty())
    }

    pub fn is_synced(&self) -> Result<bool, Box<dyn Error>> {
        if !self.is_repo_ready()? {
            println!("The repository is not ready");
            return Ok(false);
        }

        let repo_path = &self.config.load()?.repo_path;
        // fetch the latest changes from the remote
        cmd!("git -C {repo_path} fetch --all")?;

        // check if the current branch is synced with the remote
        let current_branch = cmd!("git -C {repo_path} rev-parse --abbrev-ref HEAD")?;
        let remote_branch = format!("origin/{current_branch}");
        let stdout = cmd!("git -C {repo_path} rev-list HEAD...{remote_branch}")?;

        if !stdout.is_empty() {
            return Ok(false);
        }

        Ok(self.is_contents_synced()?)
    }

    fn is_contents_synced(&self) -> Result<bool, Box<dyn Error>> {
        let repo_path = &self.config.load()?.repo_path;

        for dotfile in &self.config.load()?.dotfiles {
            let home = env!("HOME");
            let dotfile_path = format!("{home}/{dotfile}");
            let repo_dotfile_path = format!("{repo_path}/{dotfile}");

            if !std::path::Path::new(&dotfile_path).exists() {
                return Ok(false);
            }

            match std::fs::read_link(&dotfile_path) {
                Ok(link) => {
                    if link != std::path::Path::new(&repo_dotfile_path) {
                        return Ok(false);
                    }
                }
                Err(_) => {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    pub fn sync(&self) -> Result<(), Box<dyn Error>> {
        self.init_repo()?;

        let repo_path = &self.config.load()?.repo_path;

        // pull
        println!("Pulling the repository");
        cmd!("git -C {repo_path} pull")
            .map_err(|e| format!("Failed to pull the repository: {e}"))?;

        if !self.is_clean()? {
            // commit
            println!("Committing the changes");
            cmd!("git -C {repo_path} add .")?;
            cmd!("git -C {repo_path} commit -m 'Update dotfiles by Sync-dot-files'")?;

            // push
            println!("Pushing the changes");
            cmd!("git -C {repo_path} push")?;
        }

        println!("Repository is synced");

        // hard-link the dotfiles
        println!("Checking dotfiles");
        for dotfile in &self.config.load()?.dotfiles {
            println!("Checking {dotfile}");

            let home = env!("HOME");
            let dotfile_path = format!("{home}/{dotfile}");
            let repo_dotfile_path = format!("{repo_path}/{dotfile}");

            if std::path::Path::new(&dotfile_path).exists() {
                // check if the dotfile is a symlink
                match std::fs::read_link(&dotfile_path) {
                    Ok(link) => {
                        if link != std::path::Path::new(&repo_dotfile_path) {
                            println!("{dotfile} is not linked to the repository");
                        }
                    }
                    Err(e) => {
                        println!("{dotfile} is not a symlink: {e}");
                    }
                }

                continue;
            }

            println!("Linking {dotfile} from the repository");
            symlink(repo_dotfile_path, dotfile_path)?;
        }

        Ok(())
    }
}
