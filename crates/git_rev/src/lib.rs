use once_cell::sync::OnceCell;

pub static REV_FULL: OnceCell<String> = OnceCell::new();
pub static REV_10: OnceCell<String> = OnceCell::new();

pub fn get() -> String {
    REV_FULL
        .get()
        .expect("git_rev not initialized. Did you forget to call `ambient_git_rev_init::init()`?")
        .clone()
}
