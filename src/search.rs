use crate::git::Commit;
use firestorm::profile_fn;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod methods;

pub use methods::exact_diff::ExactDiffMatch;
pub use methods::lsh::TraditionalLSH;
pub use methods::message_scan::MessageScan;

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CherryAndTarget {
    cherry: CommitMetadata,
    target: CommitMetadata,
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMetadata {
    id: String,
    parent_ids: Vec<String>,
    message: String,
    author: String,
    committer: String,
    time: String,
}

impl CommitMetadata {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn author(&self) -> &str {
        &self.author
    }
    pub fn committer(&self) -> &str {
        &self.committer
    }
    pub fn time(&self) -> &str {
        &self.time
    }

    pub fn parent_ids(&self) -> &[String] {
        &self.parent_ids
    }
}

impl<'r, 'c> From<&Commit<'r, 'c>> for CommitMetadata {
    fn from(commit: &Commit) -> Self {
        Self {
            id: commit.id().to_string(),
            parent_ids: commit.parent_ids().iter().map(|p| p.to_string()).collect(),
            message: commit.message().map_or(String::new(), |m| m.to_string()),
            author: commit.author().to_string(),
            committer: commit.committer().to_string(),
            time: format!("{:?}", commit.time()),
        }
    }
}

// TODO: A commit can only be the target for a cherry-pick once? Or should the library return all possible source-target pairs?

impl CherryAndTarget {
    /// Construct a new CherryPick for two commits. Cherry and target are determined based on the commit time
    pub fn construct(commit_a: &Commit, commit_b: &Commit) -> Self {
        profile_fn!(construct);
        if commit_a.time() < commit_b.time() {
            // commit_a is older than commit_b
            Self::new(commit_a, commit_b)
        } else {
            Self::new(commit_b, commit_a)
        }
    }

    /// Create a new CherryPick with the ids of two commits for which the cherry and target relationship is known
    pub fn new(cherry: &Commit, target: &Commit) -> Self {
        Self {
            cherry: CommitMetadata::from(cherry),
            target: CommitMetadata::from(target),
        }
    }

    pub fn as_vec(&self) -> Vec<&CommitMetadata> {
        vec![&self.cherry, &self.target]
    }

    pub fn into_vec(self) -> Vec<CommitMetadata> {
        vec![self.cherry, self.target]
    }

    pub fn cherry(&self) -> &CommitMetadata {
        &self.cherry
    }

    pub fn target(&self) -> &CommitMetadata {
        &self.target
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    search_method: String,
    cherry_and_target: CherryAndTarget,
}

impl SearchResult {
    pub fn new(search_method: String, cherry_ids: CherryAndTarget) -> Self {
        Self {
            search_method,
            cherry_and_target: cherry_ids,
        }
    }

    /// The SearchMethod type that was used to find this result
    pub fn search_method(&self) -> &str {
        &self.search_method
    }

    // TODO: Have references to not break connection?
    /// The commit pair of this cherry pick. Commits are identified by their id.
    pub fn commit_pair(&self) -> &CherryAndTarget {
        &self.cherry_and_target
    }
}

/// Trait for implementing new search methods. This trait is meant to annotate the capabilities of
/// a type to function as a search search, on the one hand, and to offer a common interface for
/// search methods on the other hand.
///
/// A SearchMethod is supposed to search and find all existing cherry picks in a given slice of
/// commits. How a cherry pick is identified is left to the search search.
/// On this note, the results returned by a search search must not objectively be correct.
/// The returned set of SearchResult instances should instead be seen as possible cherry picks that
/// can be used and validated by the caller.
///
/// # Examples
/// Example of a naive search search that finds cherry picks only based on the equality of
/// commit messages.
/// ```
///  
/// use cherry_harvest::{CherryAndTarget, Commit, SearchMethod, SearchResult};
/// use std::collections::HashSet;
///
/// struct NaiveSearch();
///
/// const NAME: &str = "NaiveSearch";
///
/// impl SearchMethod for NaiveSearch {
///     fn search(&self, commits: &mut [Commit]) -> HashSet<SearchResult> {
///         let mut results: HashSet<SearchResult> = HashSet::new();
///         for commit_a in commits.iter() {
///             for commit_b in commits.iter() {
///                 // Guard against matching the same commit
///                 if commit_a.id() == commit_b.id() {
///                     continue;
///                 }
///                 // Naively determine a cherry pick as two commits having the same commit message
///                 if commit_a.message() == commit_b.message() {
///                     // Determine the order of the commits by their timestamp
///                     let cherry_pick = CherryAndTarget::construct(commit_a, commit_b);
///                     results.insert(SearchResult::new(String::from(NAME), cherry_pick));
///                 }
///             }
///         }
///         results
///     }
///
///     fn name(&self) -> &'static str {
///         "NAIVE_SEARCH"
///     }
/// }
/// ```
pub trait SearchMethod {
    /// Searches for all cherry picks in the given slice of commits.
    fn search(&self, commits: &mut [Commit]) -> HashSet<SearchResult>;

    /// The search's name that is to be stored with each SearchResult
    /// TODO: Find a better approach to handling the association of results and search methods
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use crate::search::CommitMetadata;
    use crate::{CherryAndTarget, SearchResult};
    use std::collections::HashSet;

    #[test]
    fn same_result_same_hash() {
        let create_a = || CommitMetadata {
            id: "aaa".to_string(),
            parent_ids: vec![],
            message: "aaa".to_string(),
            author: "aaa".to_string(),
            committer: "aaa".to_string(),
            time: "aaa".to_string(),
        };
        let create_b = || CommitMetadata {
            id: "aba".to_string(),
            parent_ids: vec![],
            message: "aba".to_string(),
            author: "aba".to_string(),
            committer: "aba".to_string(),
            time: "aba".to_string(),
        };

        let result_a = SearchResult {
            search_method: "TEST".to_string(),
            cherry_and_target: CherryAndTarget {
                cherry: create_a(),
                target: create_b(),
            },
        };

        let result_b = SearchResult {
            search_method: "TEST".to_string(),
            cherry_and_target: CherryAndTarget {
                cherry: create_a(),
                target: create_b(),
            },
        };

        let mut set = HashSet::new();
        set.insert(result_a);
        set.insert(result_b);

        assert_eq!(set.len(), 1);
    }
}
