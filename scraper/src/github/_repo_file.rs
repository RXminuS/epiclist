use super::{DateTime, URI};
use anyhow::{Context, Result};
use graphql_client::GraphQLQuery;
use std::borrow::Cow;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../schemas/github.graphql",
    query_path = "src/github/_repo_file.graphql",
    response_derives = "Debug,Serialize,Deserialize,Clone"
)]
pub struct RepoFileWithHistory;

impl repo_file_with_history::ResponseData {
    pub fn file_content(&self) -> Result<Cow<str>> {
        let blob = match self
            .repository
            .as_ref()
            .context("missing file repository")?
            .file
            .as_ref()
            .context("missing file")?
        {
            repo_file_with_history::RepoFileWithHistoryRepositoryFile::Blob(blob) => {
                blob.text.as_deref().clone()
            }
            _ => unreachable!(),
        }
        .unwrap_or_default();

        Ok(Cow::Borrowed(blob))
    }
}
