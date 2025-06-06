use std::path::PathBuf;

// Example custom build script.
fn main() {
    let workspace_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_owned();
    let docs_dir = {
        let mut dir = workspace_dir.clone();
        dir.push("docs");
        dir
    };
    let docs_gitignore = {
        let mut dir = docs_dir.clone();
        dir.push(".gitignore");
        dir
    };
    let bioimg_gui_dir = {
        let mut dir = workspace_dir.clone();
        dir.push("bioimg_gui");
        dir
    };

    let a = std::process::Command::new("trunk")
        .current_dir(&bioimg_gui_dir)
        .arg("build")
        .arg("--release=true")
        .arg("--public-url=https://kreshuklab.github.io/bioimageio_rs/")
        .arg(format!("--dist={}", docs_dir.to_string_lossy()))
        .output().expect("Expected trunk to run");

    //de-ignore the docs dir so that it can be committed to the gh-pages branch
    std::fs::write(&docs_gitignore, "!*").unwrap();
}
