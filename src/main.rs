use std::{env, fs};
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
        let mut input_match = Some(String::new());
        let result = match_block(&input_line, &pattern[1..], 0, &mut input_match, false);
        if let Some(im) = input_match {
            matches.push(im.clone());
        }
        result
    } else {
        let mut pad = 0;
        for (i, v) in input_line.chars().enumerate() {
            let mut input_match = Some(String::new());
            pad = pad + v.to_string().len();
            if match_block(&input_line, &pattern, i, &mut input_match, false) {
                if let Some(im) = &input_match {
                    matches.push(im.clone());
                }
                let match_len = input_match.map_or(0, |x| x.len());
                if pad + match_len < input_line.len() {
                    match_re(&input_line[pad + match_len - 1..], pattern, matches);
                }
                return true
            }
        }
        let pat = &pattern[extract_pattern(pattern).len()..];
        if input_line.is_empty() && (pat.ends_with('?') || pat.ends_with('*')) {
            matches.push("".to_string());
            true
        } else {
            false
        }
    }
}

fn match_repeats(input_line: &str, pattern: &str, pattern_ahead: &str, input_skip: usize) -> usize {
    let consumed = input_line.chars().skip(input_skip).take_while(|c| match_pattern(c.to_string().as_str(), pattern)).count();
    backtrack(input_line.chars().skip(input_skip).collect::<String>().as_str(), pattern_ahead, consumed)
}

fn backtrack(input_line: &str, pattern: &str, consumed: usize) -> usize {
    if consumed == 0 || match_block(input_line, pattern, consumed, &mut None, false) {
        consumed
    } else {
        backtrack(input_line, pattern, consumed - 1)
    }
}

fn match_zero_or_one(input_line: &str, pattern: &str, input_skip: usize, pattern_skip: usize) -> (usize, usize) {
    match input_line.chars().skip(input_skip).next() {
        Some(c) if match_pattern(c.to_string().as_str(), pattern) =>
            (input_skip + 1, pattern_skip + pattern.len() + 1),
        Some(c) if !match_pattern(c.to_string().as_str(), pattern) =>
            (input_skip, pattern_skip + pattern_skip + pattern.len() + 1),
        _ =>
            (input_skip, pattern_skip)
    }
}

fn match_block(input_line: &str, pattern: &str, skip: usize, input_match: &mut Option<String>, reset: bool) -> bool {
    if reset && let Some(s) = input_match.as_mut() {
        let str = s.chars().take(skip).collect::<String>();
        s.clear();
        s.push_str(str.as_str());
    }
    if pattern == "$" {
        input_line.chars().skip(skip).count() == 0
    } else if pattern.chars().count() > 0 {
        let single_pattern = extract_pattern(pattern);
        if (single_pattern.starts_with('(')) && single_pattern.ends_with(')') {
            single_pattern[1..single_pattern.len() - 1]
                .split_terminator('|')
                .any(|x| match_block(input_line, format!("{}{}", x, &pattern[single_pattern.len()..]).as_str(), skip, input_match, true))
        } else {
            let quantifier = pattern.chars().skip(single_pattern.chars().count()).next();
            match input_line.chars().skip(skip).next() {
                Some(c) if quantifier == Some('+') => {
                    if match_pattern(c.to_string().as_str(), single_pattern) {
                        if let Some(s) = input_match.as_mut() {
                            s.push_str(c.to_string().as_str());
                        }
                        let consumed = match_repeats(&input_line, single_pattern, &pattern[single_pattern.len() + 1..], skip+1);
                        if let Some(s) = input_match.as_mut() {
                            s.push_str(&input_line[skip + 1..skip + consumed + 1]);
                        }
                        match_block(&input_line, &pattern[single_pattern.len() + 1..], skip + consumed + 1, input_match, false)
                    } else {
                        false
                    }
                },
                Some(c) if quantifier == Some('*') => {
                    if match_pattern(c.to_string().as_str(), single_pattern) {
                        if let Some(s) = input_match.as_mut() {
                            s.push_str(c.to_string().as_str());
                        }
                        let consumed = match_repeats(&input_line, single_pattern, &pattern[single_pattern.len() + 1..], skip+1);
                        if let Some(s) = input_match.as_mut() {
                            s.push_str(&input_line[skip + 1..skip + consumed + 1]);
                        }
                        match_block(&input_line, &pattern[single_pattern.len() + 1..], skip + consumed + 1, input_match, false)
                    } else {
                        match_block(&input_line, &pattern[single_pattern.len() + 1..], skip, input_match, false)
                    }
                },
                Some(_) if quantifier == Some('?') => {
                    let (input_skip, pattern_skip) = match_zero_or_one(&input_line, single_pattern, skip, 0);
                    if let Some(s) = input_match.as_mut() {
                        s.push_str(&input_line[skip..input_skip]);
                    }
                    match_block(&input_line, &pattern[pattern_skip..], input_skip, input_match, false)
                },
                None if quantifier == Some('?') || quantifier == Some('*') => {
                    match_block(&input_line, &pattern[single_pattern.len() + 1..], skip, input_match, false)
                },
                Some(c) => {
                    if match_pattern(c.to_string().as_str(), single_pattern) {
                        if let Some(s) = input_match.as_mut() {
                            s.push_str(c.to_string().as_str());
                        }
                        match_block(&input_line, &pattern[single_pattern.len()..], skip + 1, input_match, false)
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

fn collect_files(dir: String) -> Vec<String> {
    let mut result = Vec::new();
    let dir_iter = match fs::read_dir(dir) {
        Ok(iter) => iter,
        Err(_) => return Vec::new()
    };

    result.extend(
        dir_iter
            .filter_map(|entry| entry.ok())
            .flat_map(|entry| {
                let path = entry.path().to_string_lossy().to_string();
                if entry.path().is_file() { vec![path] } else { collect_files(path) }
            })
    );

    result
}

fn main() -> io::Result<()> {
    let mut cnt = 1;
    let mut only_matching = false;
    let mut recursive = false;

    if env::args().nth(cnt).unwrap() == "-o" {
        only_matching = true;
        cnt = cnt + 1;
    }

    if env::args().nth(cnt).unwrap() == "-r" {
        recursive = true;
        cnt = cnt + 1;
    }

    if env::args().nth(cnt).unwrap() != "-E" {
        println!("Missing argument '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(cnt + 1).unwrap();
    let mut matches: Vec<String> = Vec::new();
    let files: Vec<String> = if !recursive {
        env::args().skip(cnt + 2).collect()
    } else {
        collect_files(env::args().skip(cnt + 2).next().unwrap())
    };

    let lines = if files.is_empty() && !recursive {
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
                                if files.len() > 1 || recursive {
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
        assert_eq!(match_re("45", "\\d+", &mut matches), true);
        assert_eq!(matches, vec!["45"]);
        matches = Vec::new();
        assert_eq!(match_re("pear", ".+er", &mut matches), false);
        matches = Vec::new();
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
        matches = Vec::new();
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
        assert_eq!(match_re("scala", "(swift|scala)", &mut matches), true);
        assert_eq!(matches, vec!["scala"]);
        matches = Vec::new();
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

    #[test]
    fn match_star() {
        let mut matches = Vec::new();
        assert_eq!(match_re("scal", "scala*", &mut matches), true);
        assert_eq!(matches, vec!["scal"]);
        matches = Vec::new();
        assert_eq!(match_re("bg", "ba*g", &mut matches), true);
        assert_eq!(matches, vec!["bg"]);
        matches = Vec::new();
        assert_eq!(match_re("", "a*", &mut matches), true);
        assert_eq!(matches, vec![""]);
    }

}
