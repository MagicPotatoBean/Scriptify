# Scriptify.rs
A rust-script program that takes a rust crate and converts it into a single file rust-script for nix-shell.

## Examples
```sh
$ ./scriptify.rs --root-dir ./my_crate/ --out-path ./my_crate/script.rs
```

## Known issues
- All files in "src/" will be included in the final script, even non-rust files e.g. html files.
    - This means that any non-valid rust, or non-rust data included in "src/" will prevent the script from compiling(such as html files or input files)
    - This means that any secrets in "src/" will be included in the script as a module(and if the secret is not valid rust, would make the script un-runnable)
