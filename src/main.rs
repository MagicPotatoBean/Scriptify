#!/usr/bin/env nix-shell
//! ```cargo
//! [dependencies]
//! clap = {version = "4.5.8", features = ["derive"]}
//! ```

use std::{fs::File, io::Read, path::PathBuf};

use anyhow::anyhow;
use clap::{command, Parser};
/*
#!nix-shell -i rust-script -p rustc -p rust-script -p cargo
*/
/// A rust-script to convert rust files into rust-scripts
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        verbatim_doc_comment,
        value_parser,
        num_args = 1,
        required = true
    )]
    root_dir: PathBuf,
}
type Result<T> = anyhow::Result<T>;
fn main() -> Result<()> {
    let args = Args::parse();
    let Ok(root_dir) = std::env::current_dir()
        .expect("Failed to get current directory")
        .join(args.root_dir)
        .canonicalize()
    else {
        return Err(anyhow!("No such file or directory".to_string()));
    };
    if !root_dir.is_dir() {
        return Err(anyhow!("Specified path is not a directory"));
    }
    println!("Scriptifying {}/", root_dir.display());
    println!("manifest:\n");
    println!(
        "{}",
        generate_manifest(
            std::fs::OpenOptions::new()
                .read(true)
                .open(root_dir.join("Cargo.toml"))?
        )?
    );

    Ok(())
}
fn generate_manifest(mut cargo_toml: File) -> Result<String> {
    let mut manifest = "#!/usr/bin/env nix-shell\n//! ```cargo\n".to_string();
    let mut cargo_contents = String::default();
    cargo_toml.read_to_string(&mut cargo_contents)?;
    for line in cargo_contents.lines() {
        manifest.push_str("//! ");
        manifest.push_str(line);
        manifest.push('\n');
    }
    manifest.push_str("/*\n#!nix-shell -i rust-script -p rustc -p rust-script -p cargo\n*/\n");
    /*
    #!nix-shell -i rust-script -p rustc -p rust-script -p cargo
    */
    Ok(manifest)
}
enum SrcTree {
    File(String),
    Directory(Vec<SrcTree>),
}
impl SrcTree {
    fn from_path(path: PathBuf) -> Result<Self> {
        if path.is_file() {
            let mut contents = String::default();
            std::fs::OpenOptions::new()
                .read(true)
                .open(path)?
                .read_to_string(&mut contents)?;
            return Ok(SrcTree::File(contents));
        } else {
            let mut contents = Vec::new();
            for entry in std::fs::read_dir(path)? {
                contents.push(SrcTree::from_path(entry?.path())?);
            }
            return Ok(SrcTree::Directory(contents));
        }
    }
}
fn join_src_tree(tree: SrcTree) -> Result<String> {}
