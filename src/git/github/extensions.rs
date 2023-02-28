use octocrab::models::Repository as OctoRepo;
use octocrab::{Octocrab, Page};
use reqwest::Url;

#[async_trait::async_trait]
pub trait ForksExt {
    async fn list_forks(&self, forks_url_for_repo: Url) -> octocrab::Result<Page<OctoRepo>>;
}

#[async_trait::async_trait]
impl ForksExt for Octocrab {
    async fn list_forks(&self, forks_url_for_repo: Url) -> octocrab::Result<Page<OctoRepo>> {
        self.get(forks_url_for_repo, None::<&()>).await
    }
}
