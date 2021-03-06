use last_git_commit::LastGitCommit;
use std::fs;

pub fn get_version() -> String {
    let lgc = LastGitCommit::new().set_path("./").build().unwrap();
    let cargo_version = env!("CARGO_PKG_VERSION");

    format!("{}-{}", cargo_version, lgc.id().short())
}

pub fn write_version(version: &str) {
    fs::write("out/version.txt", version).unwrap();
}
