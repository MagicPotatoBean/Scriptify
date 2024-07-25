#!/usr/bin/env nix-shell
//! ```cargo
//! [dependencies]
//! clap = {version = "4.5.8", features = ["derive"]}
//! anyhow = "1.0.86"
//! color-eyre = "0.6.3"
//! ```

/*
#!nix-shell -i rust-script -p rustc -p rust-script -p cargo
*/
use std::{fs::File, io::Read, path::PathBuf};

use anyhow::anyhow;
use clap::{command, Parser};
/// A rust-script to convert rust files into rust-scripts
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The root directory of the program being script-ified (the path containing Cargo.toml and src/)
    #[arg(short, long, value_parser, required = true)]
    root_dir: PathBuf,
    /// The path of the file to write the script to
    #[arg(short, long, value_parser, required = true)]
    out_path: PathBuf,
}
type Result<T> = anyhow::Result<T>;
pub fn main() -> Result<()> {
    let args = Args::parse();
    let Ok(root_dir) = std::env::current_dir()
        .expect("Failed to get current directory")
        .join(&args.root_dir)
        .canonicalize()
    else {
        return Err(anyhow!("No such file or directory".to_string()));
    };
    if !root_dir.is_dir() {
        return Err(anyhow!("Specified path is not a directory"));
    }
    println!("Scriptifying {}/", root_dir.display());
    let mut data = generate_manifest(
        std::fs::OpenOptions::new()
            .read(true)
            .open(root_dir.join("Cargo.toml"))?,
    )?;
    println!("Generated manifest");
    data.push_str(&join_src_tree(SrcTree::from_path(
        args.root_dir.join(PathBuf::from("src/")),
    )?)?);
    println!("Merged source files");
    let res = Ok(std::fs::write(&args.out_path, data)?);
    if res.is_ok() {
        println!(
            "Success! You may have to run\n $ chmod +x {0}\nin order to make the script executable. Then just call\n $ ./{0}\nTo execute it.",
            args.out_path.display()
        );
        Ok(())
    } else {
        println!("Failed to write final file out.");
        res
    }
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
#[derive(Debug)]
enum SrcTree {
    File {
        name: String,
        data: String,
    },
    Directory {
        name: String,
        children: Vec<SrcTree>,
    },
}
impl SrcTree {
    fn from_path(path: PathBuf) -> Result<Self> {
        if path.is_file() {
            let mut contents = String::default();
            std::fs::OpenOptions::new()
                .read(true)
                .open(&path)?
                .read_to_string(&mut contents)?;
            return Ok(SrcTree::File {
                name: path
                    .file_name()
                    .ok_or(anyhow!("Path wasnt canonicalized"))?
                    .to_string_lossy()
                    .to_string(),
                data: clean_code(contents),
            });
        } else {
            let mut contents = Vec::new();
            for entry in std::fs::read_dir(&path)? {
                contents.push(SrcTree::from_path(entry?.path())?);
            }
            return Ok(SrcTree::Directory {
                name: path
                    .file_name()
                    .ok_or(anyhow!("Path wasnt canonicalized"))?
                    .to_string_lossy()
                    .to_string(),
                children: contents,
            });
        }
    }
}
fn clean_code(code: String) -> String {
    let mut code: Vec<_> = code.lines().collect();
    code.retain(|line| !(line.starts_with("mod ") && line.ends_with(";")));
    code.retain(|line| !(line.starts_with("pub mod ") && line.ends_with(";")));
    code.join("\n")
}
fn join_src_tree(tree: SrcTree) -> Result<String> {
    match tree {
        SrcTree::File { name, data } => {
            let mut buffer;
            let should_close;
            if name == "mod.rs" || name == "main.rs" {
                buffer = "".to_string();
                should_close = false;
            } else {
                should_close = true;
                let name_chars: Vec<char> = name.chars().collect();
                buffer = format!(
                    "mod {} {{\n",
                    &name_chars[0..(name_chars.len() - 3)]
                        .iter()
                        .collect::<String>()
                );
            }
            buffer.push_str(&data);
            if should_close {
                buffer.push_str("\n}");
            }
            return Ok(buffer);
        }
        SrcTree::Directory { name, children } => {
            if name == "src" {
                let mut buffer = String::default();
                for child in children {
                    buffer.push_str(&join_src_tree(child)?);
                    buffer.push('\n');
                }
                return Ok(buffer);
            } else {
                let mut buffer = format!("mod {name} {{\n");
                for child in children {
                    buffer.push_str(&join_src_tree(child)?);
                    buffer.push('\n');
                }
                buffer.push_str("\n}\n");
                return Ok(buffer);
            }
        }
    }
}
