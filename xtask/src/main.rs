use clap::{CommandFactory, Parser};
use clap_complete::{generate_to, Generator, Shell};
use devx_cmd::{cmd, run};
use handlr_regex::Cmd;
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

type DynResult = Result<(), Box<dyn Error>>;

fn main() -> DynResult {
    match Task::parse() {
        Task::Dist => dist()?,
    }

    Ok(())
}

/// Action for `cargo xtask dist`
fn dist() -> DynResult {
    if fs::remove_dir_all(dist_dir()).is_ok() {
        eprintln!("Deleted {}", dist_dir().to_str().unwrap());
    };

    dist_binary()?;

    let mut cmd = Cmd::command();
    dist_manpage(&cmd)?;
    dist_completions(&mut cmd)
}

/// Build and strip binary
fn dist_binary() -> DynResult {
    eprintln!("Building binary");
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    eprintln!("Running cargo build --release");
    cmd!(cargo, "build", "--release")
        .current_dir(project_root())
        .run()?;

    let out_dir = dist_dir();
    fs::create_dir_all(&out_dir)?;
    let dst = project_root().join("target/release/handlr");
    fs::copy(&dst, &out_dir.join("handlr"))?;

    eprintln!("Stripping binary");
    run!("strip", &dst)?;

    Ok(())
}

/// Generate man page for binary and subcommands
fn dist_manpage(cmd: &clap::Command) -> DynResult {
    eprintln!("Generating man pages");
    generate_manpage(cmd)?;

    for sub in cmd.get_subcommands() {
        generate_manpage(sub)?
    }

    Ok(())
}

/// Generate man page for one command
fn generate_manpage(cmd: &clap::Command) -> DynResult {
    if cmd.is_hide_set() {
        return Ok(());
    }

    let old_name = cmd.get_name();
    let is_main_cmd = old_name == "handlr-regex";

    let cmd = if is_main_cmd {
        cmd.clone().name("handlr")
    } else {
        cmd.clone().name(format!("handlr-{}", old_name))
    };

    let man = clap_mangen::Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Default::default();

    // Render man page
    man.render(&mut buffer)?;

    // Add "-regex" to (sub)command name
    let buffer =
        regex::bytes::Regex::new(r"handlr(?P<name>\\-[[:alpha:]]+)? \\-")?
            .replace(&buffer, r"handlr-regex$name -".as_bytes());

    // Replace dash in subcommands' synopsis command names with a space
    let buffer =
        regex::bytes::Regex::new(r"handlr\\-(?P<name>[[:alpha:]]+)\\")?
            .replace(&buffer, r"handlr $name\".as_bytes());

    let out_dir = dist_dir().join("man1");

    // Write man page to file
    fs::create_dir_all(&out_dir)?;

    let file = if is_main_cmd {
        "handlr.1".to_string()
    } else {
        format!("handlr-{}.1", old_name)
    };

    let file = out_dir.join(file);
    fs::write(&file, buffer)?;
    eprintln!("Created {}", file.to_str().unwrap());
    Ok(())
}

/// Generate completion scripts
fn dist_completions(cmd: &mut clap::Command) -> DynResult {
    let bin_name = "handlr";
    let file = dist_dir().join(bin_name);
    generate_to(Shell::Zsh, cmd, bin_name, dist_dir())?;

    eprintln!("Created {}", file.to_str().unwrap());
    Ok(())
}

#[derive(Parser, Clone, Copy, Debug)]
enum Task {
    /// Build program and generate man page
    Dist,
}

// Project root
fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

/// Output directory for `cargo xtast dist`
fn dist_dir() -> PathBuf {
    project_root().join("target/dist")
}
