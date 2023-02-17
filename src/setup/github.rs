mod forks;

use crate::error::{Error, ErrorKind};
use crate::RepoLocation;
use chrono::{DateTime, NaiveDateTime, Utc};
use log::error;
use octocrab::models::{Repository as OctoRepo, RepositoryId};
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
    source_id: RepositoryId,
    // Maps child ids to parent ids. Only includes repos that have a parent.
    parents: HashMap<RepositoryId, RepositoryId>,
    // Maps parent ids to children ids (i.e., forks). Only includes repos that have been forked.
    forks: HashMap<RepositoryId, Vec<RepositoryId>>,
}

impl ForkNetwork {
    // TODO: test
    pub fn build_from(seed: OctoRepo) -> Self {
        let source_id;
        let mut repository_map = HashMap::new();
        let mut parent_map = HashMap::<RepositoryId, RepositoryId>::new();
        let mut children_map = HashMap::<RepositoryId, Vec<RepositoryId>>::new();

        match seed.source {
            None => {
                source_id = seed.id;
                repository_map.insert(seed.id, seed);
            }
            Some(source) => {
                source_id = source.id;
                repository_map.insert(source_id, source.as_ref().clone());
            }
        }

        let source = repository_map.get(&source_id).unwrap();

        let mut forks = retrieve_forks(source);
        while let Some(repos) = forks.as_ref() {
            let mut fork_children = vec![];
            for fork in repos {
                let fork_id = fork.id.clone();
                // Handle all forks of the fork (i.e., the forks children)
                if let Some(mut children) = retrieve_forks(fork) {
                    let children_ids: Vec<RepositoryId> =
                        children.iter().map(|c| c.id.clone()).collect();
                    // Map each child to the parent
                    for child_id in &children_ids {
                        assert!(parent_map
                            .insert(child_id.clone(), fork_id.clone())
                            .is_none());
                    }
                    // Map the parent to its children
                    assert!(children_map.insert(fork_id.clone(), children_ids).is_none());
                    // Collect children for later processing
                    fork_children.append(&mut children);
                }
                // Add the fork to the repository map
                repository_map.insert(fork_id, fork.clone());
            }

            match fork_children.is_empty() {
                true => forks = None,
                false => forks = Some(fork_children),
            }
        }

        // Convert all repos
        let repository_map = repository_map
            .into_iter()
            .map(|(k, v)| (k, GitHubRepo::from(&v)))
            .collect();

        Self {
            repositories: repository_map,
            source_id,
            parents: parent_map,
            forks: children_map,
        }
    }

    pub fn repository_ids(&self) -> Vec<RepositoryId> {
        self.repositories.keys().copied().collect()
    }

    pub fn repositories(&self) -> Vec<&GitHubRepo> {
        self.repositories.values().collect()
    }

    pub fn forks(&self, repo: &GitHubRepo) -> Option<Vec<&GitHubRepo>> {
        match self.forks.get(&repo.id) {
            None => None,
            Some(fork_ids) => fork_ids
                .iter()
                .map(|id| self.repositories.get(id))
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.repositories.len()
    }
}

fn retrieve_forks(octo_repo: &OctoRepo) -> Option<Vec<OctoRepo>> {
    let url = match &octo_repo.forks_url {
        None => return None,
        Some(url) => url.clone(),
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Retrieve the first page with forks
    let api_result: Result<Page<OctoRepo>, octocrab::Error> = runtime.block_on(forks_api(url));
    let mut page = match api_result {
        Ok(page) => page,
        Err(error) => {
            error!("{error}");
            return None;
        }
    };

    // Loop through all pages and collect all forks in them
    let mut forks = vec![];
    'breakable: loop {
        // Collect forks in current page
        for fork in &page {
            forks.push(fork.clone());
        }
        // Get the next page
        match runtime.block_on(get_page::<OctoRepo>(&page.next)) {
            Ok(Some(p)) => {
                page = p;
            }
            Ok(None) => {
                // No more pages left
                break 'breakable;
            }
            Err(error) => {
                error!("{error}");
                return Some(forks);
            }
        }
    }
    Some(forks)
}

async fn get_page<T: serde::de::DeserializeOwned>(
    url: &Option<Url>,
) -> Result<Option<Page<T>>, octocrab::Error> {
    octocrab::instance().get_page::<T>(url).await
}

use forks::ForksExt;
async fn forks_api(forks_url_for_repo: Url) -> Result<Page<OctoRepo>, octocrab::Error> {
    octocrab::instance()
        .forks()
        .list(forks_url_for_repo)
        .send()
        .await
}

pub async fn first_repo_pushed_after_datetime(
    datetime: NaiveDateTime,
) -> Result<Option<OctoRepo>, Error> {
    match search_repositories(
        format!("pushed:>{}", datetime.format("%Y-%m-%dT%H:%M:%S")).as_str(),
        1,
    )
    .await
    {
        Ok(page) => match page.items.first() {
            None => Ok(None),
            Some(octo_repo) => Ok(Some(octo_repo.clone())),
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
