use crate::awesome_links::{extract_awesome_links, AwesomeLink};
use crate::github;

use kdam::{term, term::Colorizer, tqdm, BarExt, Column, RichProgress, Spinner};
use std::collections::HashSet;
use std::collections::{vec_deque, BTreeSet};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io::Write, path::Path};

use anyhow::{Context, Result};
use clap::Args;
use clap::{builder::NonEmptyStringValueParser, Arg, ArgMatches, Command};
use fs4::FileExt;
use itertools::Itertools;
use reqwest::Client;
use tokio::sync::RwLock;

use graphql_client::reqwest::post_graphql;
use serde::{Deserialize, Serialize};

#[derive(Debug, Args)]
pub struct CrawlArgs {
    output_path: PathBuf,

    #[clap(long)]
    max_repos: Option<usize>,

    #[clap(long, env)]
    github_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrawledRepoData {
    pub crawled_at: chrono::DateTime<chrono::Utc>,
    pub repo_view: github::repo_view::ResponseData,
    pub readme: github::repo_file_with_history::ResponseData,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CrawledAwesomeList {
    pub owner: String,
    pub repo: String,
    pub description: Option<String>,
    pub crawled_at: chrono::DateTime<chrono::Utc>,
    pub latest_commit_at: chrono::DateTime<chrono::Utc>,
    pub awesome_links: Vec<AwesomeLink>,
}

impl CrawlArgs {
    pub async fn run(&self) -> Result<()> {
        let cache_path = self.output_path.join("cache");

        fs::create_dir_all(&cache_path)?;
        let md_path = self.output_path.join("mds");
        fs::create_dir_all(&md_path)?;
        let awesome_lists_path = self.output_path.join("awesome_lists");
        fs::create_dir_all(&awesome_lists_path)?;

        let client = Client::builder()
            .user_agent("graphql-rust/0.9.0")
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!(
                        "Bearer {}",
                        &self.github_token
                    ))
                    .unwrap(),
                ))
                .collect(),
            )
            .build()?;
        let mut to_scrape =
            VecDeque::from([(String::from("sindresorhus"), String::from("awesome"), true)]);

        let mut processed = HashSet::<String>::new();

        //TODO: add caching and re-fetching
        let mut pb = tqdm!();
        pb.write("Fetching awesome lists...".colorize("bold blue"))?;
        while let Some((owner, repo, follow_links)) = to_scrape.pop_front() {
            pb.update(1)?;
            let repo_data = match fetch_with_cache(&owner, &repo, &client, &cache_path).await {
                Ok(data) => data,
                Err(e) => {
                    pb.write(format!(
                        "{}: {}",
                        f!("{owner}/{repo}").colorize("bold red"),
                        e
                    ))?;
                    continue;
                }
            };
            let readme_content = repo_data.readme.file_content()?;
            let links = extract_awesome_links(&readme_content)?;

            let md_path = md_path.join(f!("{owner}-{repo}.md"));
            let mut output_file = std::fs::File::create(md_path)?;
            output_file.write_all(readme_content.as_bytes())?;

            let awesome_list = CrawledAwesomeList {
                owner: owner.clone(),
                repo: repo.clone(),
                awesome_links: links,
                description: repo_data
                    .repo_view
                    .repository
                    .as_ref()
                    .and_then(|r| r.description.clone()),
                crawled_at: repo_data.crawled_at.clone(),
                latest_commit_at: repo_data.readme.latest_commit_date()?.clone(),
            };
            let list_path = awesome_lists_path.join(f!("{owner}-{repo}.json"));
            let output_file = std::fs::File::create(list_path)?;
            let writer = std::io::BufWriter::new(output_file);
            serde_json::to_writer_pretty(writer, &awesome_list)?;

            if follow_links {
                let to_add = awesome_list
                    .awesome_links
                    .into_iter()
                    .flat_map(|link| match link.as_github_repo() {
                        Some((owner, repo)) => Some((owner.to_string(), repo.to_string(), false)),
                        _ => None,
                    })
                    .filter(|(owner, repo, _)| {
                        let cache_key = f!("{owner}/{repo}");
                        let exists = processed.contains(&cache_key);
                        if !exists {
                            processed.insert(cache_key);
                        };
                        !exists
                    });
                if self.max_repos.unwrap_or_default() > 0 {
                    to_scrape.extend(to_add.take(self.max_repos.unwrap_or_default()))
                } else {
                    to_scrape.extend(to_add)
                }
            }
        }

        Ok(())
    }
}

async fn fetch_with_cache(
    owner: &str,
    repo: &str,
    client: &Client,
    cache_path: &Path,
) -> Result<CrawledRepoData> {
    //TODO: right now this always loads a cached file first.

    let cache_key = cache_path.join(f!("{owner}-{repo}.json"));

    let read_cached = || -> Result<CrawledRepoData> {
        let rdr = std::fs::File::open(&cache_key)?;
        let out: CrawledRepoData = serde_json::from_reader(rdr)?;
        Ok(out)
    };

    if cache_key.exists() {
        return read_cached();
    }

    let root_data = post_graphql::<github::RepoView, _>(
        &client,
        "https://api.github.com/graphql",
        github::repo_view::Variables {
            name: repo.to_string(),
            owner: owner.to_string(),
        },
    )
    .await?
    .data
    .context("missing repo response data")?;

    let root_repo = root_data
        .repository
        .as_ref()
        .context("missing repository")?;
    let root_files = root_repo
        .root_files
        .as_ref()
        .context("missing root files")?;
    let readme_path = root_files.find_readme_path()?.context("no root readme")?;

    let readme_data: github::repo_file_with_history::ResponseData =
        post_graphql::<github::RepoFileWithHistory, _>(
            &client,
            "https://api.github.com/graphql",
            github::repo_file_with_history::Variables {
                ref_filename: f!("HEAD:{readme_path}"),
                filename: readme_path,
                name: repo.to_string(),
                owner: owner.to_string(),
            },
        )
        .await?
        .data
        .context("missing readme response data")?;

    let out = CrawledRepoData {
        crawled_at: chrono::Utc::now(),
        repo_view: root_data,
        readme: readme_data,
    };

    if let Some(file) = std::fs::File::create_new(&cache_key).ok() {
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &out)?;
        Ok(out)
    } else {
        //Someone beat me to it
        return read_cached();
    }
}
