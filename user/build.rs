use std::path::Path;

fn main() {
    let linker_script = Path::new("src/arch/riscv64/linker.ld");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    println!("cargo::rerun-if-changed={}", linker_script.display());
    println!(
        "cargo::rustc-link-arg-bins=-T{}",
        manifest_dir.join(linker_script).display()
    );
}
