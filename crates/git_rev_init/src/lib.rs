pub const REV_FULL: &str = env!("REV_FULL");
pub const REV_10: &str = env!("REV_10");

#[derive(Debug)]
pub struct InitError;

/// Initialise git revisions. Fails if called multiple times.
pub fn init() -> Result<(), InitError> {
    ambient_git_rev::REV_FULL
        .set(REV_FULL.into())
        .or(Err(InitError))?;
    ambient_git_rev::REV_10
        .set(REV_10.into())
        .or(Err(InitError))
}

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
