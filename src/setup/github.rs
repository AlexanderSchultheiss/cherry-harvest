mod forks;

use crate::error::{Error, ErrorKind};
use crate::RepoLocation;
use chrono::{DateTime, NaiveDateTime, Utc};
use log::{debug, error};
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
    // The maximum number of forks that this network can consist of
    max_forks: Option<usize>,
}

impl ForkNetwork {
    // TODO: test
    // TODO: Refactor to improve readability
    // TODO: Implement Display for ForkNetwork for manual verification
    pub fn build_from(seed: OctoRepo, max_forks: Option<usize>) -> Self {
        debug!("building fork network for {}:{}", seed.name, seed.id);
        let source_id;
        let mut repository_map = HashMap::new();
        let mut parent_map = HashMap::<RepositoryId, RepositoryId>::new();
        let mut children_map = HashMap::<RepositoryId, Vec<RepositoryId>>::new();

        match seed.source {
            None => {
                debug!("the repository is the source of its network");
                source_id = seed.id;
                repository_map.insert(seed.id, seed);
            }
            Some(source) => {
                debug!("found source with id {}", source.id);
                source_id = source.id;
                repository_map.insert(source_id, source.as_ref().clone());
            }
        }

        let source = repository_map.get(&source_id).unwrap();

        let runtime = tokio::runtime::Runtime::new().unwrap();

        let mut forks_retrieved = 0;
        let mut forks = runtime.block_on(retrieve_forks(source, max_forks));
        if let Some(repos) = forks.as_ref() {
            // Map the source to its children
            let children_ids: Vec<RepositoryId> = repos.iter().map(|c| c.id).collect();
            forks_retrieved = children_ids.len();
            // Map each child to the parent and vice versa
            for child_id in &children_ids {
                assert!(parent_map.insert(*child_id, source_id).is_none());
            }
            assert!(children_map.insert(source_id, children_ids).is_none());
        } else {
            debug!("there are no forks");
        }

        while let Some(repos) = forks.as_ref() {
            debug!("{} forks need to be processed...", repos.len());
            let mut fork_children = vec![];
            for fork in repos {
                let fork_id = fork.id;
                debug!("fork_id: {fork_id}");
                // Handle all forks of the fork (i.e., the forks children)
                if let Some(mut children) = runtime.block_on(retrieve_forks(
                    fork,
                    max_forks.map(|mf| mf - forks_retrieved),
                )) {
                    let children_ids: Vec<RepositoryId> = children.iter().map(|c| c.id).collect();
                    forks_retrieved += children_ids.len();
                    debug!("fork {fork_id} has {} forks of its own", children.len());
                    // Map each child to the parent
                    for child_id in &children_ids {
                        assert!(parent_map.insert(*child_id, fork_id).is_none());
                    }
                    // Map the parent to its children
                    assert!(children_map.insert(fork_id, children_ids).is_none());
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
            if Some(forks_retrieved) >= max_forks {
                break;
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
            max_forks,
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

    pub fn source(&self) -> &GitHubRepo {
        self.repositories.get(&self.source_id).unwrap()
    }
}

async fn retrieve_forks(octo_repo: &OctoRepo, max_forks: Option<usize>) -> Option<Vec<OctoRepo>> {
    match octo_repo.forks_count {
        None => return None,
        Some(num) if num == 0 => return None,
        Some(num) => debug!("discovered {num} forks"),
    }
    let url = match &octo_repo.forks_url {
        None => return None,
        Some(url) => url.clone(),
    };

    // Retrieve the first page with forks
    let api_result: Result<Page<OctoRepo>, octocrab::Error> = forks_api(url).await;
    let page = match api_result {
        Ok(page) => page,
        Err(error) => {
            error!("{error}");
            return None;
        }
    };

    // Loop through all pages and collect all forks in them
    collect_repos_from_pages(page, max_forks).await
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

pub async fn repos_created_in_time_range(
    n_repos: usize,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Result<Option<Vec<OctoRepo>>, Error> {
    let time_format = "%Y-%m-%dT%H:%M:%S+00:00";
    let query = format!(
        "created:{}..{}",
        start.format(time_format),
        end.format(time_format)
    );
    debug!("search query: '{}'", query);

    // Retrieve the first page
    let page = match search_repositories(query.as_str()).await {
        Ok(page) => page,
        Err(error) => return Err(Error::new(ErrorKind::GitHub(error))),
    };

    Ok(collect_repos_from_pages(page, Some(n_repos)).await)
}

async fn collect_repos_from_pages(
    start_page: Page<OctoRepo>,
    n_repos: Option<usize>,
) -> Option<Vec<OctoRepo>> {
    let mut page = start_page;
    let mut repos: Vec<OctoRepo> = vec![];
    'breakable: loop {
        // Collect repos in current page
        for repo in &page {
            if Some(repos.len()) == n_repos {
                break 'breakable;
            }
            repos.push(repo.clone());
        }
        // Get the next page
        match next_page(&page).await {
            None => break 'breakable,
            Some(p) => page = p,
        };
    }
    match repos.is_empty() {
        true => None,
        false => Some(repos),
    }
}

async fn next_page<T: serde::de::DeserializeOwned>(page: &Page<T>) -> Option<Page<T>> {
    match get_page::<T>(&page.next).await {
        Ok(Some(p)) => Some(p),
        Ok(None) => {
            // No more pages left
            None
        }
        Err(error) => {
            error!("{error}");
            None
        }
    }
}

pub async fn search_repositories(query: &str) -> Result<Page<OctoRepo>, octocrab::Error> {
    octocrab::instance()
        .search()
        .repositories(query)
        .send()
        .await
}

#[cfg(test)]
mod tests {}
