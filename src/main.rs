use reqwest;
use reqwest::Method;
use serde::Deserialize;
use std::error::Error;
use std::io::{self, Write};
use std::process::Command;
use std::{thread, time};

fn main() -> Result<(), Box<Error>> {
    let client = reqwest::Client::new();
    let token = "09184b2c4cf2e1a2e69e23c5515a888460eb9c65";

    let repo_urls = get_repos(client, token)?;

    let mut threads = vec![];
    for repo_url in repo_urls {
        threads.push(thread::spawn(move || {
            println!("Cloning {}...", repo_url);
            let output = Command::new("sh")
                .arg("-c")
                .arg(format!("git clone {}", repo_url))
                .output()
                .expect(format!("Failed to execute clone for {}", repo_url).as_str());
            io::stdout().write_all(&output.stdout).unwrap();
        }));
    }

    for thread in threads {
        let _ = thread.join();
    }

    Ok(())
}

fn get_repos(client: reqwest::Client, token: &str) -> Result<Vec<String>, reqwest::Error> {
    let mut url =
        Some("https://api.github.com/orgs/urbanairship/repos?page=1&per_page=100".to_string());
    let mut response;
    let mut repo_urls: Vec<String> = Vec::new();

    while url.as_ref().is_some() {
        let address = url.unwrap();
        println!("{}", address);
        response = client
            .request(Method::GET, &address)
            .bearer_auth(token)
            .send()?;

        let repos: Vec<Repo> = response.json()?;
        for repo in repos {
            if repo.private
                && !repo.archived
                && repo
                    .language
                    .as_ref()
                    .map(|l| l == "Java" || l == "Scala")
                    .unwrap_or(false)
            {
                println!(
                    "Will clone {}, language is {}",
                    repo.name,
                    repo.language.unwrap_or(String::from(""))
                );
                repo_urls.push(repo.clone_url);
            }
        }

        let link_header: String = response
            .headers()
            .get("Link")
            .expect("No link header received")
            .to_str()
            .expect("Failed to transform header to string")
            .to_string();
        url = get_next_page(link_header);
        thread::sleep(time::Duration::from_millis(10));
    }

    Ok(repo_urls)
}

fn get_next_page(link_header: String) -> Option<String> {
    let next: Vec<String> = link_header
        .split(",")
        .filter(|rel| rel.ends_with("rel=\"next\""))
        .map(|s| s.to_string())
        .collect();

    Some(
        next.get(0)
            .map(|s| s.clone())?
            .split(";")
            .collect::<Vec<&str>>()
            .get(0)?
            .trim_start()
            .trim_start_matches("<")
            .trim_end()
            .trim_end_matches(">")
            .to_string(),
    )
}

#[derive(Deserialize)]
struct Repo {
    name: String,
    private: bool,
    clone_url: String,
    language: Option<String>,
    archived: bool,
}
