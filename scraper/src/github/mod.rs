#[allow(clippy::upper_case_acronyms)]
pub type URI = String;

pub type DateTime = chrono::DateTime<chrono::Utc>;

mod _repo_file;
mod _repo_view;

pub use _repo_file::{repo_file_with_history, RepoFileWithHistory};
pub use _repo_view::{repo_view, RepoView};
