// Do not warn when `cfg(executable)` is encountered here in build script
#![allow(unexpected_cfgs)]

mod cli {
    include!("src/cli.rs");
}

use cli::Cli;

use clap::CommandFactory;
use std::{env, error::Error, fs::create_dir_all, path::Path};

type DynResult = Result<(), Box<dyn Error>>;

fn main() -> DynResult {
    // Build handlr with `cfg(executable)`
    // Needed to help with weirdness involving including modules from src/ into build script
    println!("cargo:rustc-check-cfg=cfg(executable)");
    println!("cargo:rustc-cfg=executable");

    println!("cargo:rerun-if-changed=build/");
    let out_dir = Path::new(&env::var("OUT_DIR")?).to_path_buf();
    mangen(&out_dir)
}

/// Generate man page for binary and subcommands
fn mangen(out_dir: &Path) -> DynResult {
    println!("cargo:rerun-if-env-changed=PROJECT_NAME");
    println!("cargo:rerun-if-env-changed=PROJECT_EXECUTABLE");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

    eprintln!("Generating man pages");

    let dest_dir = out_dir.join("manual/man1");
    create_dir_all(&dest_dir)?;

    clap_mangen::generate_to(Cli::command().name("handlr"), &dest_dir)?;

    Ok(())
}
