### Formatting
This repository incorporates a certain formatting style derived from the [rustfmt defaults](https://github.com/rust-lang/rustfmt/blob/master/Configurations.md) with a few changes, which are:
 - Maximum line width is 120 characters
 - Newline separator style is always Unix (`\n` as opposed to Windows `\n\r`)
 - `try!` macro would automatically convert into a `?` question-mark operator expression
 
Any other configuration is the **stable** `rustfmt` default.

#### Running the formatter
`evm-rs` is a complex project with a lot of sub-crates, so `cargo fmt` should be invoked with an `--all` argument.
```bash
cargo fmt --all
```
More info can be found at the [project page](https://github.com/rust-lang/rustfmt)

#### Manual formatting overrides
The formatting constraint is checked within Travis CI and Jenkins continuous integration pipelines, hence any pull-request should be formatted before it may be merged.

Though, as most of the tooling generally is, `rustfmt` isn’t perfect and sometimes one would requite to force the manual formatting, as, for instance, it’s required for the `Patch` interface in `evm-rs`
##### Original code
```rust
pub struct EmbeddedAccountPatch;
impl AccountPatch for EmbeddedAccountPatch {
    fn initial_nonce() -> U256 { U256::zero() }
    fn initial_create_nonce() -> U256 { Self::initial_nonce() }
    fn empty_considered_exists() -> bool { true }
}
```
##### `Rustfmt` formatted code
```rust
pub struct EmbeddedAccountPatch;
impl AccountPatch for EmbeddedAccountPatch {
    fn initial_nonce() -> U256 { 
        U256::zero() 
    }
    fn initial_create_nonce() -> U256 { 
        Self::initial_nonce() 
    }
    fn empty_considered_exists() -> bool { 
        true 
    }
}
```
Depending on the case, the readability of the code may decrease, like here (IMHO) since the expanded function body brings none value. 
While this would be possible to fix with **nightly** `rustfmt`, nightly version is still too unstable, so it’s encouraged to use manual formatting overrides where it is justified.
##### Manual override
```rust
pub struct EmbeddedAccountPatch;
#[rustfmt::skip]
impl AccountPatch for EmbeddedAccountPatch {
    fn initial_nonce() -> U256 { U256::zero() }
    fn initial_create_nonce() -> U256 { Self::initial_nonce() }
    fn empty_considered_exists() -> bool { true }
}
```
#### Automation
To ensure none commits are misformatted, it’s required to manually run rustfmt before commiting the code, though that might be irritating.
Fortunately, formatting (and formatting checks) may be automated, and here are ways to do that:
##### 1. Formatting on save
Most of the editors have a feature of pre-save hooks that can execute arbitrary commands before persisting the file contents.
* [Setup for JetBrains IDEs (Clion, Intellij Idea, …)](https://codurance.com/2017/11/26/rusting-IntelliJ/)
* [Setup for VSCode](https://github.com/editor-rs/vscode-rust/blob/master/doc/format.md)

If the editor doesn’t support the on-save hook, one could automate formatting through [cargo watch](https://github.com/passcod/cargo-watch):
```bash
cargo watch -s “cargo fmt --all”
```
##### 2. Git pre-commit hook
Create a file `.git/hooks/pre-commit` with the following contents:
```
#!/bin/sh
# Put in your Rust repository’s .git/hooks/pre-commit to ensure you never
# breaks rustfmt. 
#
# WARNING: rustfmt is a fast moving target so ensure you have the version that
# all contributors have.
for FILE in `git diff --cached --name-only`; do
    if [[ $FILE == *.rs ]] && ! rustfmt --check $FILE; then
        echo “Commit rejected due to invalid formatting of \”$FILE\" file.”
        exit 1
    fi
done
```
This hook will reject ill-formatted code before the commit.

Synergy of these two automation techniques should allow one to ensure formatting correctness while not being forced to run `rustfmt` manually.


