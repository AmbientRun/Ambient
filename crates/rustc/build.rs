fn main() {
    // Store TARGET for rustup fetch
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
