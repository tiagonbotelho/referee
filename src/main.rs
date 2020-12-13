use std::env;
use referee::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    referee::run(config)
}
