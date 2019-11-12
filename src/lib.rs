use rayon::prelude::*;

use reqwest;
use reqwest::Method;
use serde::Deserialize;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use std::{result, thread, time};

type Result<T> = result::Result<T, String>;

#[derive(Deserialize)]
struct Repo {
    name: String,
    private: bool,
    clone_url: String,
    language: Option<String>,
    archived: bool,
}

pub fn update_repos() -> Result<()> {
    let repo_dirs = get_local_repos().unwrap();

    repo_dirs.par_iter().for_each(|dir| update_local_repo(&dir));

    // for repo_dir in repo_dirs {
    //     update_local_repo(&repo_dir)
    // }

    Ok(())
}

pub fn clone_repos(token: &str) -> Result<()> {
    let client = reqwest::Client::new();

    let repo_urls = get_remote_repos(client, token).map_err(|e| e.to_string())?;

    repo_urls.par_iter().for_each(|url| clone_remote_repo(&url));

    Ok(())
}

fn clone_remote_repo(repo_url: &str) {
    println!("Cloning {}...", repo_url);
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("git clone {}", repo_url))
        .output()
        .expect(format!("Failed to execute git clone for {}", repo_url).as_str());
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();
}

fn update_local_repo(path: &PathBuf) {
    let check_branch = Command::new("sh")
        .current_dir(&path)
        .arg("-c")
        .arg("git rev-parse --abbrev-ref HEAD")
        .output()
        .expect(
            format!(
                "Failed to get current branch for directory {}",
                path.display()
            )
            .as_str(),
        );

    match check_branch.status.code() {
        Some(0) => (),
        _ => {
            println!("{} is not a Git repo, skipping...", path.display());
            return;
        }
    };

    let branch = String::from_utf8(check_branch.stdout).unwrap();
    let branch = branch.trim();

    if branch == "master" {
        println!("Updating {} to latest", path.display());
        let update = Command::new("git")
            .arg("pull")
            .current_dir(path)
            .output()
            .expect(format!("Failed to pull for directory {}", path.display()).as_str());
        io::stdout().write_all(&update.stdout).unwrap();
        io::stderr().write_all(&update.stderr).unwrap();
    } else {
        println!(
            "Directory {} is on branch {}, skipping...",
            path.display(),
            branch.trim()
        )
    }
}

fn get_local_repos() -> Result<Vec<PathBuf>> {
    let mut dirs: Vec<PathBuf> = vec![];
    let current_dir = env::current_dir().unwrap();

    for entry in fs::read_dir(current_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let metadata = fs::metadata(&path).unwrap();

        if metadata.is_dir() {
            dirs.push(path);
        }
    }

    Ok(dirs)
}

fn get_remote_repos(client: reqwest::Client, token: &str) -> Result<Vec<String>> {
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
            .send()
            .map_err(|e| e.to_string())?;

        let repos: Vec<Repo> = response.json().map_err(|e| e.to_string())?;
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
    let next: Vec<&str> = link_header
        .split(",")
        .filter(|rel| rel.ends_with("rel=\"next\""))
        .collect();

    Some(
        next.get(0)?
            .split(";")
            .collect::<Vec<&str>>()
            .get(0)?
            .trim_start()
            .trim_start_matches("<")
            .trim_end()
            .trim_end_matches(">")
            .to_owned(),
    )
}
