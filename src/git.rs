use std::{
    env,
    path::{Path, PathBuf},
};

use git2::{
    Commit, Cred, FetchOptions, Index, Oid, PushOptions, Remote, RemoteCallbacks, Repository,
    Status, StatusEntry, StatusOptions, Tree,
};

pub fn sync(path: &Path) -> Result<usize, git2::Error> {
    let repo: Repository = repo(path)?;
    println!("Using repository {:?}", repo.path());

    let mut index: Index = repo.index()?;
    let changes: usize = add_changes(&repo, &mut index)?;

    if changes > 0 {
        let commit: Oid = commit(&repo, &mut index)?;
        println!("Created commit {}", commit);
    }

    pull(&repo, "origin", "master")?
        .map(|remote| push(remote, "master"))
        .transpose()?;

    Ok(changes)
}

/// Create a handle to the existing git repository, or initailize a new repository if none is
/// present
fn repo(path: &Path) -> Result<Repository, git2::Error> {
    match Repository::open(path) {
        Ok(repo) => Ok(repo),
        Err(_) => Repository::init(path),
    }
}

/// Check for changes in tree (unstaged changes), and add them to the index
fn add_changes(repo: &Repository, mut index: &mut Index) -> Result<usize, git2::Error> {
    let mut options = StatusOptions::new();
    options.include_untracked(true).recurse_untracked_dirs(true);

    let changes: usize = repo
        .statuses(Some(&mut options))
        .unwrap()
        .iter()
        .filter(|f| filter_status(&f.status()))
        .map(|f| try_add(&mut index, f))
        .filter_map(|f| f.ok())
        .count();

    Ok(changes)
}

/// Filter out files with relevant status, i.e. files that are **not**
/// - unchanged
/// - ignored
/// - has an unresolved conflict
fn filter_status(status: &Status) -> bool {
    if status.is_conflicted() {
        panic!("File has a conflict that needs to be resolved manually")
    }
    let include = Status::all() ^ Status::CURRENT ^ Status::IGNORED ^ Status::CONFLICTED;
    status.intersects(include)
}

/// Add a created or modified file to the index
fn try_add<'a>(index: &mut Index, file: StatusEntry<'a>) -> Result<StatusEntry<'a>, git2::Error> {
    let path: PathBuf = PathBuf::from(file.path().unwrap());
    if file.status().is_wt_deleted() {
        index.remove_path(&path)?;
    } else {
        index.add_path(&path)?;
    }
    index.write_tree()?;
    Ok(file)
}

/// Commit changes that has been added to index
fn commit(repo: &Repository, index: &mut Index) -> Result<Oid, git2::Error> {
    let sign = repo.signature()?;
    let message = "Added/updated files";
    let oid: Oid = index.write_tree()?;
    let tree: Tree = repo.find_tree(oid)?;

    index.add_all(&["."], git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let parent_commit: Commit = repo.head()?.peel_to_commit()?;
    repo.commit(Some("HEAD"), &sign, &sign, message, &tree, &[&parent_commit])
}

/// Pull branch from remote
fn pull<'a>(
    repo: &'a Repository,
    remote: &str,
    branch: &str,
) -> Result<Option<Remote<'a>>, git2::Error> {
    let mut remote: Remote<'a> = match repo.find_remote(remote) {
        Ok(remote) => remote,
        Err(_) => return Ok(None),
    };
    let mut options = FetchOptions::new();
    options.remote_callbacks(callback());
    remote.fetch(&[branch], Some(&mut options), None)?;

    Ok(Some(remote))
}

/// Push commits to remote at the given branch
fn push(mut remote: Remote, branch: &str) -> Result<(), git2::Error> {
    let refspc = format!("refs/heads/{0}:refs/heads/{0}", branch);
    let refs: [&str; 1] = [&refspc];
    let mut options = PushOptions::new();
    options.remote_callbacks(callback());
    remote.push(&refs, Some(&mut options))?;

    Ok(())
}

/// Callback to handle SSH authentication
fn callback() -> RemoteCallbacks<'static> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, username, _allowed_types| {
        Cred::ssh_key(
            username.unwrap(),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
            None,
        )
    });

    cb
}
