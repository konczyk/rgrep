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
        input_line.is_empty() && pattern.ends_with('?')
    }
}

fn match_repeats(input_line: &str, pattern: &str, pattern_ahead: &str, input_skip: usize, pattern_skip: usize) -> (usize, usize) {
    let p_ahead = extract_pattern(&pattern_ahead[pattern_skip..]);
    match input_line.chars().skip(input_skip).next() {
        Some(c) if !p_ahead.is_empty() && match_pattern(c.to_string().as_str(), pattern) && !match_pattern(c.to_string().as_str(), p_ahead) =>
            match_repeats(input_line, pattern, pattern_ahead, input_skip + 1, pattern_skip),
        Some(c) if !p_ahead.is_empty() && match_pattern(c.to_string().as_str(), pattern) && match_pattern(c.to_string().as_str(), p_ahead) =>
            match_repeats(input_line, pattern, pattern_ahead, input_skip + 1, pattern_skip + p_ahead.len()),
        _ =>
            (input_skip, pattern_skip + 1)
    }
}

fn match_one_or_none(input_line: &str, pattern: &str, input_skip: usize, pattern_skip: usize) -> (usize, usize) {
    match input_line.chars().skip(input_skip).next() {
        Some(c) if match_pattern(c.to_string().as_str(), pattern) =>
            (input_skip + 1, pattern_skip + pattern.len() + 1),
        Some(c) if !match_pattern(c.to_string().as_str(), pattern) =>
            (input_skip, pattern_skip + pattern_skip + pattern.len() + 1),
        _ =>
            (input_skip, pattern_skip)
    }
}

fn match_block(input_line: &str, pattern: &str, skip: usize) -> bool {
    if pattern == "$" {
        input_line.chars().skip(skip).count() == 0
    } else if pattern.chars().count() > 0 {
        let single_pattern = extract_pattern(pattern);
        let quantifier = pattern.chars().skip(single_pattern.chars().count()).next();
        match input_line.chars().skip(skip).next() {
            Some(c) if quantifier == Some('+') => {
                let (input_skip, pattern_skip) = match_repeats(&input_line, single_pattern, &pattern[single_pattern.len() + 1..], skip + 1, 0);
                match_pattern(c.to_string().as_str(), single_pattern) && match_block(&input_line, &pattern[single_pattern.len() + pattern_skip..], input_skip)
            },
            Some(_) if quantifier == Some('?') => {
                let (input_skip, pattern_skip) = match_one_or_none(&input_line, single_pattern, skip, 0);
                match_block(&input_line, &pattern[pattern_skip..], input_skip)
            },
            None if quantifier == Some('?') => {
                match_block(&input_line, &pattern[single_pattern.len() + 1..], 0)
            },
            Some(c) =>
                match_pattern(c.to_string().as_str(), single_pattern)
                    && match_block(&input_line, &pattern[single_pattern.len()..], skip + 1),
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
    fn match_anchors() {
        assert_eq!(match_re("rust", "^r[tu]"), true);
        assert_eq!(match_re("rust", "^trust"), false);
        assert_eq!(match_re("rust", "ust$"), true);
        assert_eq!(match_re("rust", "^rust$"), true);
        assert_eq!(match_re("rust", "us$"), false);
    }

    #[test]
    fn match_combined() {
        assert_eq!(match_re("latest rust edition is 2024, it rocks", "editio\\w [big][show] \\d\\d\\d\\d[^op]"), true);
        assert_eq!(match_re("latest rust edition is 2024, it rocks", "editio\\w [big][show] \\d\\d\\d\\d[^op]"), true);
        assert_eq!(match_re("¾®_ediœ1", "\\wedi[^x]\\d"), true);
    }

    #[test]
    fn match_one_or_more() {
        assert_eq!(match_re("bag", "bag+"), true);
        assert_eq!(match_re("bag", "ba+g"), true);
        assert_eq!(match_re("bags", "ba+gs"), true);
        assert_eq!(match_re("baaag", "ba+g"), true);
        assert_eq!(match_re("baaags", "ba+gs"), true);
        assert_eq!(match_re("baag", "ba+ag"), true);
        assert_eq!(match_re("baags", "ba+ags"), true);
        assert_eq!(match_re("baaag", "ba+ag"), true);
        assert_eq!(match_re("baaags", "ba+ags"), true);
        assert_eq!(match_re("bag", "ba+ag"), false);
    }

    #[test]
    fn match_zero_or_one() {
        assert_eq!(match_re("act", "ca?t"), true);
        assert_eq!(match_re("dog", "dogs?"), true);
        assert_eq!(match_re("dogs", "dogs?"), true);
        assert_eq!(match_re("", "\\d?"), true);
        assert_eq!(match_re("5", "\\d?"), true);
        assert_eq!(match_re("dogs", "do?gs"), true);
        assert_eq!(match_re("dog", "dog?s"), false);
    }

}
