use std::fs;
use std::path::PathBuf;

pub struct GitInfo {
    pub branch: String,
    pub dirty: bool,
}

fn find_git_dir(cwd: &str) -> Option<PathBuf> {
    let mut dir = PathBuf::from(cwd);
    loop {
        let git = dir.join(".git");
        if git.is_dir() {
            return Some(git);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn read_branch(git_dir: &PathBuf) -> Option<String> {
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    if let Some(refpath) = head.strip_prefix("ref: refs/heads/") {
        Some(refpath.to_string())
    } else if head.len() >= 7 {
        Some(head[..7].to_string())
    } else {
        None
    }
}

fn index_mtime(git_dir: &PathBuf) -> Option<i64> {
    let meta = fs::metadata(git_dir.join("index")).ok()?;
    let mtime = meta.modified().ok()?;
    let duration = mtime.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(duration.as_secs() as i64)
}

pub fn read_git_info(cwd: Option<&str>, stored_mtime: Option<i64>) -> Option<(GitInfo, i64)> {
    let git_dir = find_git_dir(cwd?)?;
    let branch = read_branch(&git_dir)?;
    let current_mtime = index_mtime(&git_dir).unwrap_or(0);
    let dirty = match stored_mtime {
        Some(prev) => current_mtime != prev,
        None => false,
    };
    Some((GitInfo { branch, dirty }, current_mtime))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_git_repo(dir: &std::path::Path, head_content: &str) {
        let git_dir = dir.join(".git");
        fs::create_dir_all(git_dir.join("refs/heads")).unwrap();
        fs::write(git_dir.join("HEAD"), head_content).unwrap();
        fs::write(git_dir.join("index"), b"fake index").unwrap();
    }

    #[test]
    fn reads_branch_from_head() {
        let tmp = tempfile::tempdir().unwrap();
        setup_git_repo(tmp.path(), "ref: refs/heads/main\n");
        let (info, _) = read_git_info(Some(tmp.path().to_str().unwrap()), None).unwrap();
        assert_eq!(info.branch, "main");
    }

    #[test]
    fn reads_feature_branch() {
        let tmp = tempfile::tempdir().unwrap();
        setup_git_repo(tmp.path(), "ref: refs/heads/feat/cool-stuff\n");
        let (info, _) = read_git_info(Some(tmp.path().to_str().unwrap()), None).unwrap();
        assert_eq!(info.branch, "feat/cool-stuff");
    }

    #[test]
    fn detached_head_shows_short_hash() {
        let tmp = tempfile::tempdir().unwrap();
        setup_git_repo(tmp.path(), "abc1234def5678\n");
        let (info, _) = read_git_info(Some(tmp.path().to_str().unwrap()), None).unwrap();
        assert_eq!(info.branch, "abc1234");
    }

    #[test]
    fn not_dirty_on_first_run() {
        let tmp = tempfile::tempdir().unwrap();
        setup_git_repo(tmp.path(), "ref: refs/heads/main\n");
        let (info, _) = read_git_info(Some(tmp.path().to_str().unwrap()), None).unwrap();
        assert!(!info.dirty);
    }

    #[test]
    fn not_dirty_when_mtime_matches() {
        let tmp = tempfile::tempdir().unwrap();
        setup_git_repo(tmp.path(), "ref: refs/heads/main\n");
        let (_, mtime) = read_git_info(Some(tmp.path().to_str().unwrap()), None).unwrap();
        let (info, _) = read_git_info(Some(tmp.path().to_str().unwrap()), Some(mtime)).unwrap();
        assert!(!info.dirty);
    }

    #[test]
    fn dirty_when_mtime_differs() {
        let tmp = tempfile::tempdir().unwrap();
        setup_git_repo(tmp.path(), "ref: refs/heads/main\n");
        let (info, _) = read_git_info(Some(tmp.path().to_str().unwrap()), Some(0)).unwrap();
        assert!(info.dirty);
    }

    #[test]
    fn no_git_dir_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let result = read_git_info(Some(tmp.path().to_str().unwrap()), None);
        assert!(result.is_none());
    }

    #[test]
    fn none_cwd_returns_none() {
        assert!(read_git_info(None, None).is_none());
    }

    #[test]
    fn finds_git_in_parent() {
        let tmp = tempfile::tempdir().unwrap();
        setup_git_repo(tmp.path(), "ref: refs/heads/develop\n");
        let subdir = tmp.path().join("src/deep");
        fs::create_dir_all(&subdir).unwrap();
        let (info, _) = read_git_info(Some(subdir.to_str().unwrap()), None).unwrap();
        assert_eq!(info.branch, "develop");
    }
}
