# RGrep

Simple grep clone written in Rust

## Usage

Build project
```shell
cargo build
```

Run tests
```shell
cargo test
```

Execute
```shell
echo -n 'text' | ./target/debug/rgrep -E [PATTERN]
```
Program returns exit code 0 on match and 1 otherwise

Supported patterns
- string literals
- \d - digits
- \w - word characters
- [abc] - positive character groups
- [^abc] - negative character groups

## Examples

Match single digit
```shell
echo -n 'text9' | ./target/debug/rgrep -E '\d'
```
