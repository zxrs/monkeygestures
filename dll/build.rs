use cc;

fn main() {
    cc::Build::new()
        .cpp(true)
        .file("src\\cpp\\shareddata.cpp")
        .include("src")
        .compile("shareddata");
}