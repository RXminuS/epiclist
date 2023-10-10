use std::fs;

use anyhow::Result;
use clap::Args;

use serde::Serialize;
use sqlx::postgres::PgPoolOptions;

use super::crawl::CrawledAwesomeList;

#[derive(Debug, Args)]
pub struct IngestArgs {}

struct InsertedRes {
    id: i64,
    updated_at: chrono::NaiveDateTime,
}

impl IngestArgs {
    pub async fn run(&self) -> Result<()> {
        let input_path = std::path::Path::new("data/scrape/awesome_lists");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://postgres:password@localhost/postgres")
            .await?;

        let dir_files = fs::read_dir(input_path)?
            .flatten()
            .filter(|f| f.path().is_file());

        for dir_file in dir_files {
            let path = dir_file.path();
            let file = fs::File::open(path).unwrap();
            let Some(crawled_awesome_list) =
                serde_json::from_reader::<_, CrawledAwesomeList>(file).ok()
            else {
                continue;
            };

            sqlx::query!(
                r"--sql
                WITH project AS (
                    INSERT INTO awesome_projects (url) VALUES($1) ON CONFLICT (url)
                    DO UPDATE SET id = awesome_projects.id RETURNING id
                ),
                awesome_list AS (
                    INSERT INTO awesome_lists (project_id, updated_at)
                    SELECT id, NOW()
                    FROM project ON CONFLICT (project_id)
                    DO UPDATE SET updated_at = NOW() RETURNING id
                )
                SELECT * FROM awesome_list LIMIT 1;
                ",
                f!("https://github.com/{crawled_awesome_list.owner}/{crawled_awesome_list.repo}"),
            )
            .fetch_one(&pool)
            .await?;

            //             DELETE FROM awesome_projects;

            // WITH project AS (
            // 	INSERT INTO awesome_projects (url)
            // 		VALUES('$3') ON CONFLICT (url)
            // 		DO UPDATE SET
            // 			id = awesome_projects.id
            // 		RETURNING
            // 			id
            // ), awesome_list AS (
            // 	INSERT INTO awesome_lists (project_id, updated_at)
            // 	SELECT id, NOW() FROM project
            // 	RETURNING id
            // )
            // SELECT * FROM awesome_list LIMIT 1;
        }

        Ok(())
    }
}
