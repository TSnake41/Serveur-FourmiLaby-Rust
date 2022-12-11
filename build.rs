fn main() {
    if cfg!(external_maze_gen) {
        println!("cargo:rustc-link-search=external");
        println!("cargo:rustc-link-lib=AntMaze");
    }
}