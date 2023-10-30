use super::crawl::CrawledAwesomeList;
use anyhow::Result;
use clap::Args;
use itertools::Itertools;
use kdam::tqdm;
use normalize_url_rs::normalize_url;
use sqlx::postgres::PgPoolOptions;
use std::{fs, path::PathBuf};

lazy_static! {
    static ref NORMALIZE_URL_OPTIONS: normalize_url_rs::Options =
        normalize_url_rs::OptionsBuilder::default()
            .force_https(true)
            .default_protocol("https")
            .build()
            .unwrap();
}

#[derive(Debug, Args)]
pub struct IngestArgs {
    input_path: PathBuf,

    #[clap(long, env)]
    database_url: String,
}

impl IngestArgs {
    pub async fn run(&self) -> Result<()> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.database_url)
            .await?;

        let dir_files = fs::read_dir(&self.input_path)?
            .flatten()
            .filter(|f| f.path().is_file());

        for dir_file in tqdm!(dir_files) {
            let path = dir_file.path();
            let file = fs::File::open(path).unwrap();
            let Some(crawled_awesome_list) =
                serde_json::from_reader::<_, CrawledAwesomeList>(file).ok()
            else {
                continue;
            };

            // then we bulk insert the awesome-links
            let (urls, data): (Vec<String>, Vec<_>) = crawled_awesome_list
                .awesome_links
                .iter()
                .map(|l| {
                    let raw_url = &l.url;
                    let url = normalize_url(raw_url.as_str(), &NORMALIZE_URL_OPTIONS)
                        .expect("invalid URL");

                    let data = serde_json::json!({
                        "url": url.clone(),
                        "title": l.title.clone(),
                        "description": l.description.clone(),
                        "breadcrumbs": l.breadcrumbs.to_vec()
                    });
                    (url, data)
                })
                .sorted_by(|a, b| a.0.cmp(&b.0))
                .dedup_by(|a, b| a.0 == b.0)
                .unzip();

            let _links_count = sqlx::query!(
                r"--sql
                WITH awesome_list AS (
                    INSERT INTO awesome_lists (url,
                        crawled_at,
                        latest_commit_at)
                        VALUES($1,
                            $2,
                            $3) ON CONFLICT (url)
                        DO
                        UPDATE
                        SET
                            crawled_at = $2,
                            latest_commit_at = $3
                        RETURNING
                            id
                ),
                link_data AS (
                    SELECT
                        url,
                        title,
                        description,
                        breadcrumbs
                    FROM
                        json_to_recordset($4::json) AS b (url text,
                            title text,
                            description text,
                            breadcrumbs text [])
                ),
                awesome_links AS (
                    INSERT INTO awesome_links (awesome_list_id,
                        url,
                        title,
                        description,
                        breadcrumbs)
                SELECT
                    (SELECT id FROM awesome_list),
                    url,
                    title,
                    description,
                    breadcrumbs
                FROM
                    link_data ON CONFLICT (awesome_list_id,
                        url)
                    DO
                    UPDATE
                    SET
                        title = EXCLUDED.title,
                        description = EXCLUDED.description,
                        breadcrumbs = EXCLUDED.breadcrumbs
                    RETURNING
                        1
                )
                SELECT
                    COUNT(*) AS count
                FROM
                    awesome_links;
            ",
                f!("https://github.com/{crawled_awesome_list.owner}/{crawled_awesome_list.repo}"),
                crawled_awesome_list.crawled_at,
                crawled_awesome_list.latest_commit_at,
                serde_json::Value::Array(data)
            )
            .fetch_one(&pool)
            .await?
            .count
            .unwrap_or_default();

            //todo: expire old links & projects
            //which might be just as easy as truncating the database everytime and just ensuring
            //that any "saves" to epic-lists are not dependent on it
        }

        Ok(())
    }
}
