use std::env;
use std::fs;
use lazy_static::lazy_static;
use regex::Regex;
use error_chain::error_chain;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

lazy_static! {
    static ref ELIXIR_COMMENT: Regex = Regex::new(r#"@doc "{3}([^"{3}]*)"{3}|#(.*)"#).unwrap();
    static ref URL_REGEX: Regex = Regex::new(r#"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"#).unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];
    println!("Parsing {}...", filename);

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let processed_urls = process_file(&contents);

    // TODO: Print with color coding based off of the status code
    // return an exit status based off of the status code of URLs
    for (line_number, url, status_code) in processed_urls {
        println!("{:?}: {:?} => {:?} status code", line_number, url, status_code);
    }
}

fn process_file(contents: &str) -> Vec<(usize, String, u16)> {
    let char_length_per_line = get_char_length_per_line(&contents);
    let client = reqwest::blocking::Client::new();

    ELIXIR_COMMENT.find_iter(contents).flat_map( |mat| {
        let start_line = find_line_number(mat.start(), &char_length_per_line);

        find_urls(&mat.as_str()).iter().map( |url| {
            (start_line, String::from(url), client.get(url).send().unwrap().status().as_u16())
        }).collect::<Vec<(usize, String, u16)>>()
    }).collect::<Vec<(usize, String, u16)>>()
}

fn get_char_length_per_line(contents: &str) -> Vec<usize> {
    contents.lines().map( |line| line.len() + 1).collect()
}

// TODO: Optimize this find, we can pick up from where we've left off instead
// of starting all over!
fn find_line_number(pos: usize, char_length_per_line: &Vec<usize>) -> usize {
    let mut acc: usize = 0;
    let mut current_line: usize = 0;

    for (line_number, line_length) in char_length_per_line.iter().enumerate() {
        if acc >= pos { break; }

        current_line = line_number;
        acc += line_length;
    }

    current_line + 1
}

fn find_urls(comment: &str) -> Vec<String> {
    URL_REGEX.find_iter(comment).map( |url| {
        String::from(url.as_str())
    }).collect()
}
