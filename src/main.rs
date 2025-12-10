use std::env;
use std::io;
use std::process;

fn extract_pattern(pattern_block: &str) -> &str {
    if pattern_block.chars().count() > 1 && (pattern_block.starts_with("\\d") || pattern_block.starts_with("\\w"))  {
        &pattern_block[..2]
    } else if pattern_block.starts_with("[") && pattern_block.chars().count() > 1 {
        let len = pattern_block.chars().take_while(|x| *x != ']').count()+1;
        &pattern_block[..len]
    } else if pattern_block.chars().count() > 0 {
        &pattern_block[..1]
    } else {
        pattern_block
    }
}

fn match_re(input_line: &str, pattern: &str) -> bool {
    let p = extract_pattern(pattern);
    if pattern.starts_with('^') {
        match_block(&input_line, &pattern[1..], 0)
    } else {
        for (i, v) in input_line.chars().enumerate() {
            if match_pattern(v.to_string().as_str(), &p) && match_block(&input_line, &pattern, i) {
                return true
            }
        }
        false
    }
}

fn match_block(input_line: &str, pattern: &str, skip: usize) -> bool {
    if pattern.chars().count() > 0 {
        let p = extract_pattern(pattern);
        match input_line.chars().skip(skip).next() {
            Some(c) => match_pattern(c.to_string().as_str(), p) && match_block(&input_line, &pattern[p.len()..], skip+1),
            None => false
        }
    } else {
        true
    }
}

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

    if match_re(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_single_literal() {
        assert_eq!(match_pattern("rust", "u"), true);
        assert_eq!(match_pattern("rust", "a"), false);
    }

    #[test]
    fn match_single_digit() {
        assert_eq!(match_pattern("rust23p", "\\d"), true);
        assert_eq!(match_pattern("rust", "\\d"), false);
    }

    #[test]
    fn match_single_word_char() {
        assert_eq!(match_pattern("rust", "\\w"), true);
        assert_eq!(match_pattern("123", "\\w"), true);
        assert_eq!(match_pattern("_", "\\w"), true);
        assert_eq!(match_pattern("[", "\\w"), false);
    }

    #[test]
    fn match_single_group() {
        assert_eq!(match_pattern("rust", "[rs]"), true);
        assert_eq!(match_pattern("[]", "[rs]"), false);
        assert_eq!(match_pattern("rust", "[del]"), false);
    }

    #[test]
    fn match_single_group_neg() {
        assert_eq!(match_pattern("rust", "[^rs]"), true);
        assert_eq!(match_pattern("[]", "[^rs]"), true);
        assert_eq!(match_pattern("rust", "[^rstu]"), false);
    }

    #[test]
    fn match_literals() {
        assert_eq!(match_re("rust", "ust"), true);
        assert_eq!(match_re("rust", "usta"), false);
    }

    #[test]
    fn match_digits() {
        assert_eq!(match_re("rust123", "\\d\\d\\d"), true);
        assert_eq!(match_re("rust123", "\\d\\d\\d\\d"), false);
    }

    #[test]
    fn match_word_chars() {
        assert_eq!(match_re("rust", "\\w\\w"), true);
        assert_eq!(match_re("123", "\\w\\w\\w"), true);
        assert_eq!(match_re("r", "\\w\\w"), false);
    }

    #[test]
    fn match_groups() {
        assert_eq!(match_re("rust", "[rs][at]"), true);
        assert_eq!(match_re("rust", "[rs][ab]j"), false);
    }

    #[test]
    fn match_groups_neg() {
        assert_eq!(match_re("rust", "[^ru][^ab]"), true);
        assert_eq!(match_re("rust", "[^ru][^at]"), false);
    }

    #[test]
    fn match_start_anchor() {
        assert_eq!(match_re("rust", "^r[tu]"), true);
        assert_eq!(match_re("rust", "^trust"), false);
    }

    #[test]
    fn match_combined() {
        assert_eq!(match_re("latest rust edition is 2024, it rocks", "editio\\w [big][show] \\d\\d\\d\\d[^op]"), true);
        assert_eq!(match_re("latest rust edition is 2024, it rocks", "editio\\w [big][show] \\d\\d\\d\\d[^op]"), true);
        assert_eq!(match_re("¾®_ediœ1", "\\wedi[^x]\\d"), true);
    }

}
