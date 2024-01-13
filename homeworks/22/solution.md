## Notes
Some error occurs during the installation of `cargo-risczero`, I've tried to create a report in the official repo: https://github.com/risc0/risc0/issues/1294. Apparently, they don't support Windows. Let's proceed with WSL then.

## Initialization

Install RISC Zero
```bash
cargo install cargo-binstall
cargo binstall cargo-risczero
cargo risczero install
```

Pull the example
```bash
cargo risczero new hello-world --guest-name hello_guest
``` 

Run with
```bash
cargo run
```