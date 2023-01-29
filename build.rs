fn main() {
    match () {
        #[cfg(feature = "external_maze_gen")]
        () => {
            let path = cmake::build("external");

            println!("cargo:rustc-link-search={}", path.display());
            //println!("cargo:rustc-link-lib=c++");
            //println!("cargo:rustc-link-lib=c++abi");
            println!("cargo:rustc-link-lib=projetfourmis");
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        #[cfg(not(feature = "external_maze_gen"))]
        () => {}
    }
}
