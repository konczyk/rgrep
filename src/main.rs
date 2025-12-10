use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        input_line.contains(pattern)
    } else if pattern == "\\d" {
        input_line.chars().any(|x| x.is_ascii_digit())
    } else if pattern == "\\w" {
        input_line.chars().any(|x| x.is_ascii_alphabetic() || x.is_ascii_digit() || x == '_')
    } else if pattern.starts_with("[^") && pattern.ends_with(']') {
        let pat = &pattern[2..pattern.len() - 1];
        input_line
            .chars()
            .any(|x| !pat.contains(x))
    } else if pattern.starts_with('[') && pattern.ends_with(']') {
        pattern[1..pattern.len() - 1]
            .chars()
            .any(|x| input_line.contains(x))
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Missing first argument '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_literal() {
        assert_eq!(match_pattern("rust", "u"), true);
        assert_eq!(match_pattern("rust", "a"), false);
    }

    #[test]
    fn match_digit() {
        assert_eq!(match_pattern("rust23p", "\\d"), true);
        assert_eq!(match_pattern("rust", "\\d"), false);
    }

    #[test]
    fn match_word() {
        assert_eq!(match_pattern("rust", "\\w"), true);
        assert_eq!(match_pattern("123", "\\w"), true);
        assert_eq!(match_pattern("_", "\\w"), true);
        assert_eq!(match_pattern("[", "\\w"), false);
    }

    #[test]
    fn match_group() {
        assert_eq!(match_pattern("rust", "[rs]"), true);
        assert_eq!(match_pattern("[]", "[rs]"), false);
        assert_eq!(match_pattern("rust", "[del]"), false);
    }

    #[test]
    fn match_group_neg() {
        assert_eq!(match_pattern("rust", "[^rs]"), true);
        assert_eq!(match_pattern("[]", "[^rs]"), true);
        assert_eq!(match_pattern("rust", "[^rstu]"), false);
    }
}
