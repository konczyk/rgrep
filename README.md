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
Program prints matching lines and returns exit code 0 or returns exit code 1 otherwise

Supported patterns
- string literals
- \d - digits
- \w - word characters
- [abc] - positive character groups
- [^abc] - negative character groups
- ^ - start of string anchor
- $ - end of string anchor
- \+ - one or more times 
- \? - zero or one times
- \. - wildcard
- \| - alternation

## Examples

Match single digit
```shell
$ echo -n 'text9' | ./target/debug/rgrep -E '\d'
text9
```

Match 2 digits followed by word character, string literal and a character group
```shell
echo -n '¼more78_asone' | ./target/debug/rgrep -E '\d\d\was[done]'
```

Match exact word
```shell
$ echo -n 'rust' | ./target/debug/rgrep -E '^rust$'
rust
```

Match one or more times 
```shell
$ echo -n 'ruuust' | ./target/debug/rgrep -E '^ru+ust$'
ruuust
```

Match alternations
```shell
$ echo -n 'I love rust' | ./target/debug/rgrep -E 'I love (r?us[tv]|scala)$'
I love rust
```

Match multiple lines
```shell
$ echo -ne 'rust1\nscala2\nphp' | ./target/debug/rgrep -E '\d'
rust1
scala2
```
