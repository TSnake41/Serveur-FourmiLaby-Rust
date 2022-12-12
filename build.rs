fn main() {
    match () {
        #[cfg(feature = "external_maze_gen")]
        () => {
            let path = cmake::build("external");

            println!("cargo:rustc-link-search={}", path.display());
            println!("cargo:rustc-link-lib=projetfourmis");
        },
        #[cfg(not(feature = "external_maze_gen"))]
        () => {}
    }
}
