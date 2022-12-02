use crate::git::CommitData;
use crate::method::SearchMethod;
use crate::{CommitPair, SearchResult};
use std::collections::HashSet;

#[derive(Default)]
/// The MessageScan method identifies cherry picks based on the automatically created text in a commit message.
///
/// If a cherry pick is done with the *-x* option (i.e., `git cherry-pick -x SOME_HASH`), git will
/// insert the text `(cherry picked from commit SOME_HASH)` into the commit message.
///
/// This method exploits these auto-generated message text for cherry pick recognition. First,
/// it searches the commit message of each commit for the text *'(cherry picked from commit '*.
/// If it finds the text in a commit message, it extracts the hash of the cherry-picked commit.
/// Lastly, it initializes a *SearchResult* for the commit whose message contained the text and the commit
/// identified by the extracted hash.
///
/// Under the assumption that commit messages have not been corrupted with invalid
/// *(cherry picked from...)* text deliberately, this method will only return correct results.
/// However, the method cannot guarantee to find all cherry picks, because the commit message text
/// is only generated if developers specify the *-x* option while using
/// `git cherry-pick`. Thus, the method cannot find cherry picks that were done without the option,
/// or that were done manually (i.e., copy-paste).  
pub struct MessageScan();

const NAME: &str = "MessageScan";

impl SearchMethod for MessageScan {
    fn search(&self, commits: &[CommitData]) -> HashSet<SearchResult> {
        let search_str = "(cherry picked from commit ";
        commits
            .iter()
            .filter_map(|c| {
                if let Some(index) = c.message().find(search_str) {
                    let index = index + search_str.len();
                    if let Some(end_index) = c.message()[index..].find(')') {
                        // we have to increase the end_index by the number of bytes that were cut off through slicing
                        let end_index = end_index + index;
                        let cherry_id = String::from(&c.message()[index..end_index]);
                        return Some(SearchResult::new(
                            String::from(NAME),
                            // Pair of Source-Target
                            CommitPair(cherry_id, String::from(c.id())),
                        ));
                    }
                }
                None
            })
            .collect()
    }
}
