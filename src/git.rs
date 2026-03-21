pub struct GitInfo {
    pub branch: String,
    pub dirty: bool,
}

pub fn read_git_info(_cwd: Option<&str>) -> Option<GitInfo> {
    None
}
