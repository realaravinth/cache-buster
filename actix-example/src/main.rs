use cache_buster::Files;

fn main() {
    let files = Files::load();

    assert!(file_exists("../dist/log-out.svg", &files));
    assert!(file_exists(
        "../dist/a/b/c/d/s/d/svg/credit-card.svg",
        &files
    ));

    assert!(!file_exists("dist/log-out.svg", &files));
    assert!(!file_exists("dist/a/b/c/d/s/d/svg/credit-card.svg", &files));
}

fn file_exists(path: &str, files: &Files) -> bool {
    use std::path::Path;

    if let Some(file) = files.get(path) {
        Path::new(file).exists()
    } else {
        false
    }
}
