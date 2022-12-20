use crate::git::CommitData;
use crate::SearchResult;
use std::collections::HashSet;

pub mod ann;
pub mod diff_based;
pub mod metadata_based;

pub use diff_based::diff_ann::ANNMatch;
pub use diff_based::diff_exact::ExactDiffMatch;
pub use diff_based::diff_similarity::SimilarityDiffMatch;
pub use metadata_based::message_scan::MessageScan;

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
/// use cherry_harvest::{CommitPair, SearchMethod, SearchResult};
///
/// struct NaiveSearch();
///
/// const NAME: &str = "NaiveSearch";
///
/// impl SearchMethod for NaiveSearch {
///     fn search(&self, commits: &[cherry_harvest::CommitData]) -> HashSet<SearchResult> {
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
///                     let cherry_pick = if commit_a.time() < commit_b.time() {
///                         CommitPair(commit_a.id().to_string(), commit_b.id().to_string())
///                     } else {
///                         CommitPair(commit_b.id().to_string(), commit_a.id().to_string())
///                     };
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
///     use cherry_harvest::{CommitData, CommitDiff};
///     let commit_a = CommitData::new("012ABC324".to_string(),
///                                     "Hello World!".to_string(),
///                                     CommitDiff::empty(),
///                                     "Alice".to_string(),
///                                     "Alice".to_string(),
///                                     Time::new(0, 0));
///     let commit_b = CommitData::new("883242A".to_string(),
///                                     "Hello World!".to_string(),
///                                     CommitDiff::empty(),
///                                     "Alice".to_string(),
///                                     "Bob".to_string(),
///                                     Time::new(1, 0));
///     let commits = vec![commit_a, commit_b];
///     let results = NaiveSearch().search(&commits);
///     assert_eq!(results.len(), 1);
///     results.iter().map(|r| r.commit_pair()).for_each(|p| {
///         assert_eq!(&p.0, commits[0].id());
///         assert_eq!(&p.1, commits[1].id());
///     })
/// }
/// ```
pub trait SearchMethod {
    /// Searches for all cherry picks in the given slice of commits.
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult>;

    /// The search's name that is to be stored with each SearchResult
    /// TODO: Find a better approach to handling the association of results and search methods
    fn name(&self) -> &'static str;
}
