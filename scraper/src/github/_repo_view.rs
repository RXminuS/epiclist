use super::{DateTime, URI};
use anyhow::{anyhow, Context, Result};
use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../schemas/github.graphql",
    query_path = "src/github/_repo_view.graphql",
    response_derives = "Debug"
)]
pub struct RepoView;

impl repo_view::RepoViewRepositoryRootFiles {
    pub fn find_readme_path(&self) -> Result<Option<String>> {
        let tree = match self {
            repo_view::RepoViewRepositoryRootFiles::Tree(tree) => tree,
            _ => unreachable!("root files should be a tree"),
        };

        dbg!(&tree);

        Ok(tree
            .entries
            .as_ref()
            .context("missing file entries")?
            .iter()
            .find_map(|f| match f.name.to_lowercase().as_str() {
                "readme.md" => Some(f.name.clone()),
                _ => None,
            }))
    }
}
