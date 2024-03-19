use crate::git::Commit;
use crate::search::SearchMethod;
use crate::{CherryAndTarget, SearchResult};
use firestorm::profile_method;
use git2::Oid;
use log::debug;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// MessageScan identifies cherry picks based on the automatically created text in a commit message.
///
/// If a cherry pick is done with the *-x* option (i.e., `git cherry-pick -x SOME_HASH`), git will
/// insert the text `(cherry picked from commit SOME_HASH)` into the commit message.
///
/// This search exploits these auto-generated message text for cherry pick recognition. First,
/// it searches the commit message of each commit for the text *'(cherry picked from commit '*.
/// If it finds the text in a commit message, it extracts the hash of the cherry-picked commit.
/// Lastly, it initializes a *SearchResult* for the commit whose message contained the text and the commit
/// identified by the extracted hash.
///
/// Under the assumption that commit messages have not been corrupted with invalid
/// *(cherry picked from...)* text deliberately, this search will only return correct results.
/// However, the search cannot guarantee to find all cherry picks, because the commit message text
/// is only generated if developers specify the *-x* option while using
/// `git cherry-pick`. Thus, the search cannot find cherry picks that were done without the option,
/// or that were done manually (i.e., copy-paste).  
#[derive(Default)]
pub struct MessageScan();

const NAME: &str = "MessageScan";

impl SearchMethod for MessageScan {
    fn search(&self, commits: &mut [Commit]) -> HashSet<SearchResult> {
        profile_method!(search);
        let start = Instant::now();
        let mut commit_map = HashMap::with_capacity(commits.len());
        commits.iter().for_each(|c| {
            commit_map.insert(c.id(), c);
        });

        let search_str = "(cherry picked from commit ";
        let results: HashSet<SearchResult> = commits
            .iter()
            .filter_map(|c| {
                if let Some(index) = c.message().map(|m| m.find(search_str)).flatten() {
                    let index = index + search_str.len();
                    let message = c.message().unwrap();
                    if let Some(end_index) = message[index..].find(')') {
                        // we have to increase the end_index by the number of bytes that were cut off through slicing
                        let end_index = end_index + index;
                        if let Some(cherry) =
                            commit_map.get(&Oid::from_str(&message[index..end_index]).unwrap())
                        {
                            return Some(SearchResult::new(
                                String::from(NAME),
                                // Pair of Source-Target
                                CherryAndTarget::new(cherry, c),
                            ));
                        }
                    }
                }
                None
            })
            .collect();
        debug!("found {} results in {:?}", results.len(), start.elapsed());
        results
    }

    fn name(&self) -> &'static str {
        NAME
    }
}
