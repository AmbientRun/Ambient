pub const REV_FULL: &str = env!("REV_FULL");
pub const REV_10: &str = env!("REV_10");

// This is a hack to get cargo to rebuild when the git log changes
// Also generating the path in build.rs so it's platform independent (include_bytes! expects a platform-specific path!)
const _JUST_DEP: &[u8] = include_bytes!(env!("GIT_LOG_HEAD"));

#[test]
fn test_git_rev_10() {
    let revision = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "--short=10", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .expect("Invalid encoding");
    assert_eq!(REV_10, revision.trim());
}

#[test]
fn test_git_rev_full() {
    let revision = String::from_utf8(
        std::process::Command::new("git")
            .args(["rev-parse", "--short=40", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .expect("Invalid encoding");
    assert_eq!(REV_FULL, revision.trim());
}
