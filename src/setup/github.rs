use crate::error::{Error, ErrorKind};
use crate::RepoLocation;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use octocrab::models::{Repository as OctoRepo, Repository, RepositoryId};
use octocrab::Page;
use reqwest::Url;
use std::collections::HashMap;

pub struct GitHubRepo {
    id: RepositoryId,
    name: String,
    location: RepoLocation,
    n_branches: Option<u32>,
    n_commits: Option<u32>,
    n_authors: Option<u32>,
    n_languages: Option<u32>,
    n_forks: Option<u32>,
    n_stars: Option<u32>,
    main_language: Option<String>,
    languages: Option<Vec<String>>,
    creation_date: Option<DateTime<Utc>>,
    last_updated: Option<DateTime<Utc>>,
    last_pushed: Option<DateTime<Utc>>,
}

impl From<&OctoRepo> for GitHubRepo {
    fn from(octo_repo: &OctoRepo) -> Self {
        GitHubRepo {
            id: octo_repo.id,
            name: octo_repo.name.clone(),
            location: RepoLocation::Server(octo_repo.url.to_string()),
            main_language: octo_repo.language.as_ref().map(|v| v.to_string()),
            n_stars: octo_repo.stargazers_count,
            creation_date: octo_repo.created_at,
            last_updated: octo_repo.updated_at,
            last_pushed: octo_repo.pushed_at,
            n_forks: octo_repo.forks_count,
            // TODO: retrieve missing values
            n_branches: None,
            n_commits: None,
            n_authors: None,
            n_languages: None,
            languages: None,
        }
    }
}

// TODO: we want to consider entire fork networks
// This means that we have to first collect the entire for network for a repository
// An element in the sample is then a ForkNetwork, not just a single commit!
pub struct ForkNetwork {
    repositories: HashMap<RepositoryId, GitHubRepo>,
    // The id of the repository at the root of the network
    source: RepositoryId,
    // Maps child ids to parent ids. Only includes repos that have a parent.
    parents: HashMap<RepositoryId, RepositoryId>,
    // Maps parent ids to children ids (i.e., forks). Only includes repos that have been forked.
    forks: HashMap<RepositoryId, Vec<RepositoryId>>,
}

impl ForkNetwork {
    // TODO: test
    fn build_from(seed: GitHubRepo) -> Self {
        todo!()
    }

    fn repository_ids(&self) -> Vec<RepositoryId> {
        self.repositories.keys().copied().collect()
    }
}

pub async fn first_repo_pushed_after_datetime(
    datetime: NaiveDateTime,
) -> Result<Option<GitHubRepo>, Error> {
    match search_repositories(
        format!("pushed:>{}", datetime.format("%Y-%m-%dT%H:%M:%S")).as_str(),
        1,
    )
    .await
    {
        Ok(page) => match page.items.first() {
            None => Ok(None),
            Some(octo_repo) => Ok(Some(GitHubRepo::from(octo_repo))),
        },
        Err(error) => Err(Error::new(ErrorKind::GitHub(error))),
    }
}

pub async fn search_repositories(
    query: &str,
    results_per_page: u8,
) -> Result<Page<OctoRepo>, octocrab::Error> {
    octocrab::instance()
        .search()
        .repositories(query)
        .per_page(results_per_page)
        .send()
        .await
}

#[cfg(test)]
mod tests {
    use futures_util::TryStreamExt;
    use octocrab::models::Repository as GHRepo;
    use octocrab::repos::RepoHandler;
    use octocrab::{FromResponse, Page};
    use std::error::Error;
    use std::fs;
    use tokio::pin;

    #[test]
    fn repo_search() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let octocrab = octocrab::instance();
            let page = octocrab::instance()
                .search()
                .repositories("pushed:>2013-02-01")
                .per_page(1)
                .sort("stars")
                .order("desc")
                .send()
                .await
                .unwrap();
            // let response = octocrab
            //     ._get(
            //         "https://api.github.com/repositories",
            //         Some(&[("sort", "?")]),
            //     )
            //     .await
            //     .unwrap();
            // let page: Page<GHRepo> = Page::from_response(response).await.unwrap();
            let item = &page.items[0];
            println!("{:#?}", page.items[0]);
        });
    }
}
