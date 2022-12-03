fn main() {
  println!("cargo:rustc-link-search=external");
  println!("cargo:rustc-link-lib=AntMaze");
}