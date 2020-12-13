use std::fs;
use std::fmt;
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

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Config {
        let filename = args[1].clone();

        Config { filename }
    }
}

struct UrlLineResult {
    start_line: usize,
    url: String,
    status_code: u16,
}

impl UrlLineResult {
    fn new(start_line: usize, url: &str) -> UrlLineResult {
        let status_code = reqwest::blocking::get(url).unwrap().status().as_u16();

        UrlLineResult { start_line, url: String::from(url), status_code }
    }

    fn parse_file(filename: &str) -> Vec<UrlLineResult> {
        let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
        let char_length_per_line = get_char_length_per_line(&contents);

        ELIXIR_COMMENT.find_iter(&contents).flat_map( |mat| {
            let start_line = find_line_number(mat.start(), &char_length_per_line);

            find_urls(&mat.as_str()).iter().map( |url| { 
                UrlLineResult::new(start_line, &url) 
            }).collect::<Vec<UrlLineResult>>()
        }).collect::<Vec<UrlLineResult>>()
    }

}

impl fmt::Display for UrlLineResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}: {} -> {}", self.start_line, self.url, self.status_code)
    }
}

pub fn run(config: Config) -> () {
    println!("Parsing {}...", config.filename);

    let processed_urls = UrlLineResult::parse_file(&config.filename);

    // TODO: Print with color coding based off of the status code
    // return an exit status based off of the status code of URLs
    for url_line_result in processed_urls { println!("{}", url_line_result); }

    ()
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
