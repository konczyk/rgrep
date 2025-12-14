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
$ ./target/debug/rgrep --help
Usage: rgrep [OPTIONS] -E <PATTERN> [FILE]...

Arguments:
  [FILE]...  

Options:
  -o            Print matched substring instead of matched lines
  -r            Search files recursively
  -E <PATTERN>  
  -h, --help    Print help
```
Program prints matching lines and returns exit code 0 or returns exit code 1 otherwise

#### Supported patterns:
- string literals
- \d - digits
- \w - word characters
- [abc] - positive character groups
- [^abc] - negative character groups
- ^ - start of string anchor
- $ - end of string anchor
- \+ - one or more times 
- \* - zero or more times
- \? - zero or one times
- \. - wildcard
- \| - alternation
- {n} - exactly n times 
- {n,} - at least n times
- {n,m} - at least n and at most m times
- \n - backreferences

## Examples

Match single digit
```shell
$ echo -n 'text9' | ./target/debug/rgrep -E '\d'
text9
```

Match 2 digits followed by word character, string literal and a character group
```shell
$ echo -n '¼more78_asone' | ./target/debug/rgrep -E '\d\d\was[done]'
¼more78_asone
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
Match one or more times
```shell
$ echo -n 'rust' | ./target/debug/rgrep -E '^ru*st$'
rust
```

Match alternations
```shell
$ echo -n 'I love rust' | ./target/debug/rgrep -E 'I love (r?us[tv]|scala)$'
I love rust
```

Match exactly n times
```shell
$ echo -ne 'rust123' | ./target/debug/rgrep -E '\d{2}'
rust123
```

Match at least n times
```shell
$ echo -ne 'rust123' | ./target/debug/rgrep -o -E '\d{2,}'
123
```

Match with backreferences
```shell
$ echo -ne 'rust is great and great is scala' | ./target/debug/rgrep -o -E '(\w+) and \1'
great and great
```

Match multiple lines
```shell
$ echo -ne 'rust1\nscala2\nphp' | ./target/debug/rgrep -E '\d'
rust1
scala2
```

Match multiple lines but print matches only
```shell
$ echo -ne 'rust1\nscala2\nphp' | ./target/debug/rgrep -o -E '\d'
1
2
```

Match lines from a file
```shell
$ ./target/debug/rgrep -E 's' data/file1.txt 
rust1
scala2
```

Match lines from multiple files
```shell
$ ./target/debug/rgrep -E 's' data/file1.txt data/file2.txt 
data/file1.txt:rust1
data/file1.txt:scala2
data/file2.txt:rust1
```

Match lines from multiple files by search through directory
```shell
$ ./target/debug/rgrep -r -E 's' data
data/file1.txt:rust1
data/file1.txt:scala2
data/file2.txt:rust1
```
