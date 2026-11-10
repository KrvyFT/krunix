use std::{
    env,
    ffi::OsStr,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

fn main() {
    let linker_script = Path::new("src/arch/riscv64/linker.ld");
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!("cargo::rerun-if-changed={}", linker_script.display());
    println!(
        "cargo::rustc-link-arg-bin=krunix=-T{}",
        manifest_dir.join(linker_script).display()
    );

    let source_dir = manifest_dir.join("../user/src/bin");
    println!("cargo::rerun-if-changed={}", source_dir.display());
    println!("cargo::rerun-if-env-changed=KRUNIX_APP_BIN_DIR");
    let bin_dir = env::var_os("KRUNIX_APP_BIN_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            manifest_dir
                .join("../target")
                .join(env::var("TARGET").unwrap())
                .join("release")
        });

    let mut apps: Vec<_> = fs::read_dir(source_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension() == Some(OsStr::new("rs")))
        .collect();
    apps.sort();

    let mut asm =
        String::from(".section .rodata.apps,\"a\"\n.balign 8\n.global _num_app\n_num_app:\n");
    writeln!(asm, "    .quad {}", apps.len()).unwrap();
    for index in 0..apps.len() {
        writeln!(asm, "    .quad app_{index}_start").unwrap();
    }
    asm.push_str("    .quad apps_end\n");

    for (index, source) in apps.iter().enumerate() {
        let binary = bin_dir
            .join(source.file_name().unwrap())
            .with_extension("bin");
        println!("cargo::rerun-if-changed={}", binary.display());
        assert!(
            binary.is_file(),
            "missing {}; run make user first",
            binary.display()
        );
        writeln!(asm, "app_{index}_start:\n    .incbin {:?}", binary).unwrap();
    }
    asm.push_str("apps_end:\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("link_app.S"),
        asm,
    )
    .unwrap();
}
