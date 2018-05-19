#!/usr/bin/env run-cargo-script
//! ```cargo
//! [dependencies]
//! reqwest = "*"
//! failure = "*"
//! url = "*"
//! regex = "*"
//! rayon = "*"
//! ```

extern crate reqwest;
#[macro_use] extern crate failure;
extern crate url;
extern crate regex;
extern crate rayon;

use failure::Error;
use regex::Regex;
use rayon::prelude::*;
use std::io::prelude::*;
use std::fs::File;

fn fetch_github_readme(login: &str, repo: &str) -> Result<String, Error> {
    let fetch_lowercase = || -> Result<String, Error> {
        let client = reqwest::Client::new();
        let mut res = client.get(
            &format!("https://github.com/{}/{}/blob/master/readme.md?raw=1", login, repo)
        ).send()?;

        if !res.status().is_success() && !res.status().is_redirection() {
            bail!("Invalid status code");
        }
        
        Ok(res.text()?)
    };

    let fetch_uppercase = || -> Result<String, Error> {
        let client = reqwest::Client::new();
        Ok(client.get(
            &format!("https://github.com/{}/{}/blob/master/README.md?raw=1", login, repo)
        ).send()?.text()?)
    };
    
    eprintln!("Fetching {}/{} readme...", login, repo);
    fetch_lowercase().or_else(|_| fetch_uppercase())
}

fn main() -> Result<(), Error> {
    let md = fetch_github_readme("sindresorhus", "awesome")?;

    let re_links = Regex::new(r#"\bgithub.com/(?P<login>[^/\s]+)/(?P<repo>[^/\s]+)\b"#)?;

    let links = re_links.captures_iter(&md)
        .map(|link| (
            link["login"].to_string(),
            link["repo"].to_string(),
        ))
        .collect::<Vec<_>>();

    let res = links
        .par_iter()
        .map(|(login, repo)| {
            (login, repo, fetch_github_readme(login, repo))
        })
        .filter(|res| {
            if res.2.is_err() {
                eprintln!(" - not found: {}/{}", res.0, res.1);
            }
            res.2.is_ok()
        })
        .for_each(|(login, repo, data)| {
            let path = format!("{}_{}_README.md", login, repo);
            eprintln!("Writing {}...", path);
            let _ = File::create(path).unwrap().write(data.unwrap().as_bytes());
        });
    
    eprintln!("done.");
    
    Ok(())
}