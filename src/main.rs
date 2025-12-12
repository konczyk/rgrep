use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::process;

fn extract_pattern(pattern_block: &str) -> &str {
    if pattern_block.chars().count() > 1 && (pattern_block.starts_with("\\d") || pattern_block.starts_with("\\w"))  {
        &pattern_block[..2]
    } else if pattern_block.starts_with("[") && pattern_block.chars().count() > 1 {
        let len = pattern_block.chars().take_while(|x| *x != ']').count()+1;
        &pattern_block[..len]
    } else if pattern_block.starts_with("(") && pattern_block.chars().count() > 1 {
        let len = pattern_block.chars().take_while(|x| *x != ')').count()+1;
        &pattern_block[..len]
    } else if pattern_block.chars().count() > 0 {
        &pattern_block[..1]
    } else {
        pattern_block
    }
}

fn match_re(input_line: &str, pattern: &str, matches: &mut Vec<String>) -> bool {
    if pattern.starts_with('^') {
        let mut input_match = String::new();
        let result = match_block(&input_line, &pattern[1..], 0, &mut input_match);
        matches.push(input_match.clone());
        result
    } else {
        let mut pad = 0;
        for (i, v) in input_line.chars().enumerate() {
            let mut input_match = String::new();
            pad = pad + v.to_string().len();
            if match_block(&input_line, &pattern, i, &mut input_match) {
                matches.push(input_match.clone());
                if pad + input_match.len() < input_line.len() {
                    match_re(&input_line[pad + input_match.len()-1..], pattern, matches);
                }
                return true
            }
        }
        if input_line.is_empty() && pattern[extract_pattern(pattern).len()..].ends_with('?') {
            matches.push("".to_string());
            true
        } else {
            false
        }
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

fn match_block(input_line: &str, pattern: &str, skip: usize, input_match: &mut String) -> bool {
    if pattern == "$" {
        input_line.chars().skip(skip).count() == 0
    } else if pattern.chars().count() > 0 {
        let single_pattern = extract_pattern(pattern);
        if (single_pattern.starts_with('(')) && single_pattern.ends_with(')') {
            single_pattern[1..single_pattern.len() - 1]
                .split_terminator('|')
                .any(|x| match_block(input_line, format!("{}{}", x, &pattern[single_pattern.len()..]).as_str(), skip, input_match))
        } else {
            let quantifier = pattern.chars().skip(single_pattern.chars().count()).next();
            match input_line.chars().skip(skip).next() {
                Some(c) if quantifier == Some('+') => {
                    let (input_skip, pattern_skip) = match_repeats(&input_line, single_pattern, &pattern[single_pattern.len() + 1..], skip + 1, 0);
                    input_match.push_str(&input_line[skip..input_skip]);
                    match_pattern(c.to_string().as_str(), single_pattern)
                        && match_block(&input_line, &pattern[single_pattern.len() + pattern_skip..], input_skip, input_match)
                },
                Some(_) if quantifier == Some('?') => {
                    let (input_skip, pattern_skip) = match_one_or_none(&input_line, single_pattern, skip, 0);
                    input_match.push_str(&input_line[skip..input_skip]);
                    match_block(&input_line, &pattern[pattern_skip..], input_skip, input_match)
                },
                None if quantifier == Some('?') => {
                    match_block(&input_line, &pattern[single_pattern.len() + 1..], skip, input_match)
                },
                Some(c) => {
                    if match_pattern(c.to_string().as_str(), single_pattern) {
                        input_match.push_str(c.to_string().as_str());
                        match_block(&input_line, &pattern[single_pattern.len()..], skip + 1, input_match)
                    } else {
                        false
                    }
                },
                None => false
            }
        }
    } else {
        true
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern == "." {
        true
    } else if pattern.chars().count() == 1 {
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

fn process_lines<R: BufRead>(reader: R, pattern: &str, mut matches: &mut Vec<String>) -> Vec<String> {
    reader.lines()
        .filter_map(|line| line.ok())
        .filter(|line| match_re(line.as_str(), &pattern, &mut matches))
        .collect()
}

fn main() -> io::Result<()> {
    let mut cnt = 1;
    let mut only_matching = false;

    if env::args().nth(cnt).unwrap() == "-o" {
        only_matching = true;
        cnt = cnt + 1;
    }

    if env::args().nth(cnt).unwrap() != "-E" {
        println!("Missing argument '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(cnt + 1).unwrap();
    let mut matches: Vec<String> = Vec::new();
    let files: Vec<String> = env::args().skip(cnt + 2).collect();

    let lines = if files.is_empty() {
        process_lines(BufReader::new(io::stdin().lock()), &pattern, &mut matches)
    } else {
        let mut result = Vec::new();
        for filename in &files {
            match File::open(&filename) {
                Ok(file) => {
                    let lines: Vec<String> = process_lines(BufReader::new(file), &pattern, &mut matches);
                    if !lines.is_empty() {
                        result.extend(lines
                            .iter()
                            .map(|line|
                                if files.len() > 1 {
                                    format!("{}:{}", filename, line)
                                } else {
                                    format!("{}", line)
                                }
                            )
                            .collect::<Vec<String>>()
                        );
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        result
    };

    if !lines.is_empty() {
        if only_matching {
            matches.iter().for_each(|x| println!("{}", x));
        } else {
            lines.iter().for_each(|x| println!("{}", x));
        }
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
        let mut matches = Vec::new();
        assert_eq!(match_re("rust", "ust", &mut matches), true);
        assert_eq!(matches, vec!["ust"]);
        assert_eq!(match_re("rust", "usta", &mut matches), false);
    }

    #[test]
    fn match_digits() {
        let mut matches = Vec::new();
        assert_eq!(match_re("rust123", "\\d\\d\\d", &mut matches), true);
        assert_eq!(matches, vec!["123"]);
        assert_eq!(match_re("rust123", "\\d\\d\\d\\d", &mut matches), false);
    }

    #[test]
    fn match_word_chars() {
        let mut matches = Vec::new();
        assert_eq!(match_re("rust", "\\w\\w", &mut matches), true);
        assert_eq!(matches, vec!["ru", "st"]);
        matches = Vec::new();
        assert_eq!(match_re("123", "\\w\\w\\w", &mut matches), true);
        assert_eq!(matches, vec!["123"]);
        assert_eq!(match_re("r", "\\w\\w", &mut matches), false);
    }

    #[test]
    fn match_groups() {
        let mut matches = Vec::new();
        assert_eq!(match_re("rust", "[rs][at]", &mut matches), true);
        assert_eq!(matches, vec!["st"]);
        assert_eq!(match_re("rust", "[rs][ab]j", &mut matches), false);
    }

    #[test]
    fn match_groups_neg() {
        let mut matches = Vec::new();
        assert_eq!(match_re("rust", "[^ru][^ab]", &mut matches), true);
        assert_eq!(matches, vec!["st"]);
        assert_eq!(match_re("rust", "[^ru][^at]", &mut matches), false);
    }

    #[test]
    fn match_anchors() {
        let mut matches = Vec::new();
        assert_eq!(match_re("rust", "^r[tu]", &mut matches), true);
        assert_eq!(matches, vec!["ru"]);
        matches = Vec::new();
        assert_eq!(match_re("rust", "ust$", &mut matches), true);
        assert_eq!(matches, vec!["ust"]);
        matches = Vec::new();
        assert_eq!(match_re("rust", "^rust$", &mut matches), true);
        assert_eq!(matches, vec!["rust"]);
        assert_eq!(match_re("rust", "^trust", &mut matches), false);
        assert_eq!(match_re("rust", "us$", &mut matches), false);
    }

    #[test]
    fn match_combined() {
        let mut matches = Vec::new();
        assert_eq!(match_re("latest rust edition is 2024, it rocks", "editio\\w [big][show] \\d\\d\\d\\d[^op]", &mut matches), true);
        assert_eq!(matches, vec!["edition is 2024,"]);
        matches = Vec::new();
        assert_eq!(match_re("¾®_ediœ1", "\\wedi[^x]\\d", &mut matches), true);
        assert_eq!(matches, vec!["_ediœ1"]);
    }

    #[test]
    fn match_one_or_more() {
        let mut matches = Vec::new();
        assert_eq!(match_re("bag", "bag+", &mut matches), true);
        assert_eq!(matches, vec!["bag"]);
        matches = Vec::new();
        assert_eq!(match_re("bag", "ba+g", &mut matches), true);
        assert_eq!(matches, vec!["bag"]);
        matches = Vec::new();
        assert_eq!(match_re("bags", "ba+gs", &mut matches), true);
        assert_eq!(matches, vec!["bags"]);
        matches = Vec::new();
        assert_eq!(match_re("baaag", "ba+g", &mut matches), true);
        assert_eq!(matches, vec!["baaag"]);
        matches = Vec::new();
        assert_eq!(match_re("baaags", "ba+gs", &mut matches), true);
        assert_eq!(matches, vec!["baaags"]);
        matches = Vec::new();
        assert_eq!(match_re("baag", "ba+ag", &mut matches), true);
        assert_eq!(matches, vec!["baag"]);
        matches = Vec::new();
        assert_eq!(match_re("baags", "ba+ags", &mut matches), true);
        assert_eq!(matches, vec!["baags"]);
        matches = Vec::new();
        assert_eq!(match_re("baaag", "ba+ag", &mut matches), true);
        assert_eq!(matches, vec!["baaag"]);
        matches = Vec::new();
        assert_eq!(match_re("baaags", "ba+ags", &mut matches), true);
        assert_eq!(matches, vec!["baaags"]);
        assert_eq!(match_re("bag", "ba+ag", &mut matches), false);
    }

    #[test]
    fn match_zero_or_one() {
        let mut matches = Vec::new();
        assert_eq!(match_re("act", "ca?t", &mut matches), true);
        assert_eq!(matches, vec!["ct"]);
        matches = Vec::new();
        assert_eq!(match_re("dog", "dogs?", &mut matches), true);
        assert_eq!(matches, vec!["dog"]);
        matches = Vec::new();
        assert_eq!(match_re("dogs", "dogs?", &mut matches), true);
        assert_eq!(matches, vec!["dogs"]);
        matches = Vec::new();
        assert_eq!(match_re("", "\\d?", &mut matches), true);
        assert_eq!(matches, vec![""]);
        matches = Vec::new();
        assert_eq!(match_re("5", "\\d?", &mut matches), true);
        assert_eq!(matches, vec!["5"]);
        matches = Vec::new();
        assert_eq!(match_re("dogs", "do?gs", &mut matches), true);
        assert_eq!(matches, vec!["dogs"]);
        assert_eq!(match_re("dog", "dog?s", &mut matches), false);
    }

    #[test]
    fn match_wildcard() {
        let mut matches = Vec::new();
        assert_eq!(match_re("a", ".", &mut matches), true);
        assert_eq!(matches, vec!["a"]);
        matches = Vec::new();
        assert_eq!(match_re("", ".?", &mut matches), true);
        assert_eq!(matches, vec![""]);
        matches = Vec::new();
        assert_eq!(match_re("cat", "c.t", &mut matches), true);
        assert_eq!(matches, vec!["cat"]);
        matches = Vec::new();
        assert_eq!(match_re("rust", "ru.?[abt]", &mut matches), true);
        assert_eq!(matches, vec!["rust"]);
        assert_eq!(match_re("rust", "rus.?t", &mut matches), false);
    }

    #[test]
    fn match_or() {
        let mut matches = Vec::new();
        assert_eq!(match_re("rust", "(rust|scala)", &mut matches), true);
        assert_eq!(matches, vec!["rust"]);
        matches = Vec::new();
        assert_eq!(match_re("rust", "(rus|scala)t", &mut matches), true);
        assert_eq!(matches, vec!["rust"]);
        matches = Vec::new();
        assert_eq!(match_re("rust", "(rus|scala)t?", &mut matches), true);
        assert_eq!(matches, vec!["rust"]);
        matches = Vec::new();
        assert_eq!(match_re("rust", "(r?[au]s|scala)t?", &mut matches), true);
        assert_eq!(matches, vec!["rust"]);
    }

}
