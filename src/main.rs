
use std::env;
use std::process;
use ua_cloner::clone_repos;
use ua_cloner::update_repos;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: ua-cloner <clone|update>");
        process::exit(1);
    }

    let command = &args[1];

    let result = match command.to_ascii_lowercase().as_ref() {
        "clone" => clone_repos(),
        "update" => update_repos(),
        c => Err(format!("Unknown command {}", c))
    };

    match result {
        Ok(_) => process::exit(0),
        Err(e) => eprintln!("{}", e)
    }
}