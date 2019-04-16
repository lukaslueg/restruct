fn main() {
    cc::Build::new()
        .file("src/example.c")
        .compile("example_mod")
}
