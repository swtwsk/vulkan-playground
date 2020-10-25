use std::process::Command;

// TODO: Works only on Bash shells
fn main() {
    println!("cargo:rerun-if-changed=shaders/shader.frag");
    println!("cargo:rerun-if-changed=shaders/shader.vert");

    if cfg!(windows) {
        return;
    } else {
        Command::new("shaders/compile.sh")
            .output()
            .expect("Failed to run compile script");
    }
}
