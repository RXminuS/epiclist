#[macro_use]
extern crate fstrings;

#[macro_use]
extern crate derive_builder;

mod awesome_links;
mod github;
mod parser;

use std::{collections::linked_list, fs, io::Write};

use ::reqwest::blocking::Client;
use anyhow::{anyhow, Context, Result};
use dotenv::dotenv;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};
use parser::Annotation;

fn main() -> Result<()> {
    let output_path = std::path::Path::new("data/scrape");
    fs::create_dir_all(output_path)?;

    dotenv().ok();
    let github_api_token = std::env::var("GITHUB_TOKEN").expect("missing GITHUB_TOKEN");

    let client = Client::builder()
        .user_agent("graphql-rust/0.9.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", github_api_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()?;

    let mut to_scrape = vec![(String::from("sindresorhus"), String::from("awesome"), true)];

    //TODO: add caching and re-fetching
    while let Some((owner, repo, follow_links)) = to_scrape.pop() {
        let root_data = post_graphql::<github::RepoView, _>(
            &client,
            "https://api.github.com/graphql",
            github::repo_view::Variables {
                name: repo.to_string(),
                owner: owner.to_string(),
            },
        )?
        .data
        .context("missing repo response data")?;

        dbg!(&owner, &repo);
        let root_repo = root_data.repository.context("missing repository")?;
        let root_files = root_repo.root_files.context("missing root files")?;
        let readme_path = root_files
            .find_readme_path()?
            .context("no root readme found")?;

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
            )?
            .data
            .context("missing readme response data")?;

        let readme_content = readme_data.file_content()?;
        let links = awesome_links::extract_awesome_links(&readme_content)?;

        let md_path = output_path.join(f!("{owner}-{repo}.md"));
        let mut output_file = std::fs::File::create(md_path)?;
        output_file.write_all(readme_content.as_bytes())?;

        let links_path = output_path.join(f!("{owner}-{repo}.json"));
        let output_file = std::fs::File::create(links_path)?;
        let writer = std::io::BufWriter::new(output_file);
        serde_json::to_writer_pretty(writer, &links)?;

        if follow_links {
            to_scrape.extend(
                links
                    .into_iter()
                    .flat_map(|link| match link.as_github_repo() {
                        Some((owner, repo)) => Some((owner.to_string(), repo.to_string(), false)),
                        _ => None,
                    })
                    .take(5),
            )
        }
    }

    Ok(())
}
