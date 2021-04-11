use cache_buster::Files;

fn main() {
    let files = Files::new();

    assert!(get_full_path_runner("../dist/log-out.svg", &files));
    assert!(get_full_path_runner(
        "../dist/a/b/c/d/s/d/svg/credit-card.svg",
        &files
    ));

    assert!(!get_full_path_runner("dist/log-out.svg", &files));
    assert!(!get_full_path_runner(
        "dist/a/b/c/d/s/d/svg/credit-card.svg",
        &files
    ));
}

fn get_full_path_runner(path: &str, files: &Files) -> bool {
    use std::path::Path;

    if let Some(file) = files.get_full_path(path) {
        Path::new(file).exists()
    } else {
        false
    }
}
