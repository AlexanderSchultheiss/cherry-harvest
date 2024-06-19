mod extensions;

use crate::error::{Error, ErrorKind};
use crate::git::github::extensions::ForksExt;
use crate::git::GitRepository;
use chrono::NaiveDateTime;
use http::Uri;
use log::{debug, error};
use octocrab::models::{Repository as OctoRepo, RepositoryId};
use octocrab::Page;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::RequestCooldown;

/// A ForkNetwork comprises repositories that are connected through parent-child relationships
/// depending on whether one repo has been forked from the other. The network has the following
/// properties:
/// * Each network has a source repository
/// * The source repository has no parent and can be deemed the original repository (i.e., the first one)
/// * Each repository may at most have one parent and may have an arbitrary number of children
/// * A network is a connected, directed, and acyclic graph.
/// * A network consists of at least one repository: the source repository
pub struct ForkNetwork {
    repositories: HashMap<RepositoryId, GitRepository>,
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
    /// Build a ForkNetwork that only contains the given repository.
    pub fn single(repo: OctoRepo) -> Self {
        let source_id = repo.id;
        let mut repositories = HashMap::new();
        repositories.insert(source_id, GitRepository::from(repo));
        Self {
            repositories,
            source_id,
            parents: HashMap::new(),
            forks: HashMap::new(),
            max_forks: Some(1),
        }
    }

    // TODO: test
    // TODO: Refactor to improve readability
    /// Build a new ForkNetwork for the given repository by searching GitHub for all its forks.
    ///
    /// * seed: A repository on GitHub
    /// * max_forks: The maximum number of forks in the network that should be retrieved (if desired)
    pub async fn build_from(seed: OctoRepo, max_forks: Option<usize>) -> Self {
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

        let mut forks_retrieved = 0;
        let mut forks = retrieve_forks(source, max_forks).await;
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
                // Handle all forks of the fork (i.e., the forks children)
                if let Some(mut children) =
                    retrieve_forks(fork, max_forks.map(|mf| mf - forks_retrieved)).await
                {
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
            .map(|(k, v)| (k, GitRepository::from(v)))
            .collect();

        Self {
            repositories: repository_map,
            source_id,
            parents: parent_map,
            forks: children_map,
            max_forks,
        }
    }

    /// Returns the ids of all repositories in the network in arbitrary order
    pub fn repository_ids(&self) -> Vec<RepositoryId> {
        self.repositories.keys().copied().collect()
    }

    /// Returns all references to all repositories in the network in arbitrary order
    pub fn repositories(&self) -> Vec<&GitRepository> {
        self.repositories.values().collect()
    }

    /// Returns the references to the forks of the given repository in arbitrary order
    pub fn forks(&self, repo: &GitRepository) -> Option<Vec<&GitRepository>> {
        match self.forks.get(&repo.id) {
            None => None,
            Some(fork_ids) => fork_ids
                .iter()
                .map(|id| self.repositories.get(id))
                .collect(),
        }
    }

    /// Returns the number of repositories in the network.
    pub fn len(&self) -> usize {
        self.repositories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the source repository.
    pub fn source(&self) -> &GitRepository {
        self.repositories.get(&self.source_id).unwrap()
    }
}

impl Display for ForkNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let source = self.repositories.get(&self.source_id).unwrap();

        fn write_children(
            f: &mut Formatter<'_>,
            network: &ForkNetwork,
            start: &GitRepository,
            format_text: &str,
        ) -> std::fmt::Result {
            writeln!(
                f,
                "{}- {}: {}/{}",
                format_text,
                start.id,
                start
                    .octorepo
                    .as_ref()
                    .map(|o| &o.owner.as_ref().unwrap().login)
                    .unwrap(),
                start.name
            )?;
            if let Some(children) = network.forks(start) {
                for child in children {
                    write_children(f, network, child, &format!("  {format_text}"))?;
                }
            }
            Ok(())
        }

        write_children(f, self, source, "")
    }
}

static STATIC_COOLDOWN_INSTANCE: Lazy<arc_swap::ArcSwap<Mutex<RequestCooldown>>> =
    Lazy::new(|| arc_swap::ArcSwap::from_pointee(Mutex::new(RequestCooldown::default())));

fn cooldown_instance() -> Arc<Mutex<RequestCooldown>> {
    STATIC_COOLDOWN_INSTANCE.load().clone()
}

/// Retrieves the forks for the given repository. This function collects forks until all forks have
/// been retrieved or until the specified maximum number of forks has been retrieved, if one has been
/// provided.
async fn retrieve_forks(octo_repo: &OctoRepo, max_forks: Option<usize>) -> Option<Vec<OctoRepo>> {
    match octo_repo.forks_count {
        None => return None,
        Some(0) => return None,
        Some(num) => debug!("discovered {num} forks"),
    }
    let url = match &octo_repo.forks_url {
        None => return None,
        Some(url) => url.clone(),
    };

    // Retrieve the first page with forks
    debug!("retrieve_forks");
    let gh = cooldown_instance();
    // Lock the global cooldown tracker until the request completed
    let mut gh_lock = gh.lock().await;
    gh_lock.wait_for_global_cooldown().await;

    let api_result: Result<Page<OctoRepo>, octocrab::Error> =
        octocrab::instance().list_forks(url).await;
    // drop the lock after the request
    drop(gh_lock);
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

/// Retrieve a single repository that was created in the given time range,
pub async fn repos_created_in_time_range(
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Result<Option<OctoRepo>, Error> {
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

    let repos = collect_repos_from_pages(page, Some(1))
        .await
        .and_then(|mut v| v.pop());

    Ok(repos)
}

/// Collects repositories by iterating over all pages until `max_repos` repositories have been
/// collected.
pub async fn collect_repos_from_pages(
    start_page: Page<OctoRepo>,
    max_repos: Option<usize>,
) -> Option<Vec<OctoRepo>> {
    let mut page = start_page;
    let mut repos: Vec<OctoRepo> = vec![];
    'breakable: loop {
        // Collect repos in current page
        for repo in &page {
            if Some(repos.len()) == max_repos {
                break 'breakable;
            }
            repos.push(repo.clone());
        }
        // Get the next page
        match next_page(&page.next).await {
            None => break 'breakable,
            Some(p) => page = p,
        };
    }
    match repos.is_empty() {
        true => None,
        false => Some(repos),
    }
}

pub async fn search_query(
    query: &str,
    sort: &str,
    order: &str,
    results_per_page: u8,
) -> Result<Page<OctoRepo>, octocrab::Error> {
    // Lock the global cooldown tracker until the request completed
    let gh = cooldown_instance();
    let mut gh_lock = gh.lock().await;
    gh_lock.wait_for_global_cooldown().await;
    octocrab::instance()
        .search()
        .repositories(query)
        .sort(sort)
        .order(order)
        .per_page(results_per_page)
        .page(0u32)
        .send()
        .await
}

/// Retrieves the next page for the given page
pub async fn next_page<T: serde::de::DeserializeOwned>(page: &Option<Uri>) -> Option<Page<T>> {
    match get_page::<T>(page).await {
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

/// Retrieves the page found at the given URL, if any is present.
pub async fn get_page<T: serde::de::DeserializeOwned>(
    url: &Option<Uri>,
) -> Result<Option<Page<T>>, octocrab::Error> {
    debug!("get_page");
    // Lock the global cooldown tracker until the request completed
    let gh = cooldown_instance();
    let mut gh_lock = gh.lock().await;
    gh_lock.wait_for_global_cooldown().await;

    octocrab::instance().get_page::<T>(url).await
}

pub async fn search_repositories(query: &str) -> Result<Page<OctoRepo>, octocrab::Error> {
    debug!("search_repositories");
    // Lock the global cooldown tracker until the request completed
    let gh = cooldown_instance();
    let mut gh_lock = gh.lock().await;
    gh_lock.wait_for_global_cooldown().await;

    octocrab::instance()
        .search()
        .repositories(query)
        .send()
        .await
}

// pub async fn check_search_limit(&self) -> Result<(), octocrab::Error> {
//     let limit = self.octocrab.ratelimit().get().await?;
//     let search_limit = limit.resources.search;
//     if search_limit.remaining < 2 {
//         info!(
//             "GitHub API search rate remaining: {}",
//             search_limit.remaining
//         );
//         info!("rate limit too low; waiting for reset");
//         // The search API is the limiting factor. It resets every minute.
//         time::sleep(Duration::from_secs(60)).await;
//     }
//     Ok(())
// }
