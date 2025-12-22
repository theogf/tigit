fn main() {
    // On Windows, libgit2 needs advapi32 for security/registry functions
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
