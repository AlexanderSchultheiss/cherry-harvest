use crate::git::Commit;
use firestorm::profile_fn;
use std::collections::HashSet;

mod methods;
mod util;

pub use methods::exact_diff::ExactDiffMatch;
pub use methods::message_scan::MessageScan;
pub use methods::similar_diff::ANNMatch;
pub use methods::similar_diff::HNSWSearch;
pub use methods::similar_diff::BruteForceMatch;
pub use methods::similar_diff::SimilarityDiffMatch;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct CherryAndTarget {
    cherry: String,
    target: String,
}

// TODO: A commit can only be the target for a cherry-pick once? Or should the library return all possible source-target pairs?

impl CherryAndTarget {
    /// Construct a new CherryPick for two commits. Cherry and target are determined based on the commit time
    pub fn construct(commit_a: &Commit, commit_b: &Commit) -> Self {
        profile_fn!(construct);
        if commit_a.time() < commit_b.time() {
            // commit_a is older than commit_b
            Self {
                cherry: String::from(commit_a.id()),
                target: String::from(commit_b.id()),
            }
        } else {
            Self {
                cherry: String::from(commit_b.id()),
                target: String::from(commit_a.id()),
            }
        }
    }

    /// Create a new CherryPick with the ids of two commits for which the cherry and target relationship is known
    pub fn new(cherry: String, target: String) -> Self {
        Self { cherry, target }
    }

    pub fn as_vec(&self) -> Vec<&String> {
        vec![&self.cherry, &self.target]
    }

    pub fn into_vec(self) -> Vec<String> {
        vec![self.cherry, self.target]
    }

    pub fn cherry(&self) -> &str {
        &self.cherry
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
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
/// use std::collections::HashSet;
/// use cherry_harvest::{CherryAndTarget, SearchMethod, SearchResult};
///
/// struct NaiveSearch();
///
/// const NAME: &str = "NaiveSearch";
///
/// impl SearchMethod for NaiveSearch {
///     fn search(&self, commits: &[cherry_harvest::Commit]) -> HashSet<SearchResult> {
///         let mut results: HashSet<SearchResult> = HashSet::new();
///         for commit_a in commits {
///             for commit_b in commits {
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
///
/// fn main() {
///     use git2::Time;
///     use cherry_harvest::{Commit, Diff};
///     let commit_a = Commit::new("012ABC324".to_string(),
///                                     "Hello World!".to_string(),
///                                     Diff::empty(),
///                                     "Alice".to_string(),
///                                     "Alice".to_string(),
///                                     Time::new(0, 0));
///     let commit_b = Commit::new("883242A".to_string(),
///                                     "Hello World!".to_string(),
///                                     Diff::empty(),
///                                     "Alice".to_string(),
///                                     "Bob".to_string(),
///                                     Time::new(1, 0));
///     let commits = vec![commit_a, commit_b];
///     let results = NaiveSearch().search(&commits);
///     assert_eq!(results.len(), 1);
///     results.iter().map(|r| r.commit_pair()).for_each(|p| {
///         assert_eq!(p.cherry(), commits[0].id());
///         assert_eq!(p.target(), commits[1].id());
///     })
/// }
/// ```
pub trait SearchMethod {
    /// Searches for all cherry picks in the given slice of commits.
    fn search(&self, commits: &[Commit]) -> HashSet<SearchResult>;

    /// The search's name that is to be stored with each SearchResult
    /// TODO: Find a better approach to handling the association of results and search methods
    fn name(&self) -> &'static str;
}
