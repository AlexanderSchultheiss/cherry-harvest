use crate::git::CommitData;
use crate::method::SearchMethod;
use crate::SearchResult;

#[derive(Default)]
pub struct MessageScan();

const NAME: &str = "MessageScan";

impl SearchMethod for MessageScan {
    fn search(&self, commits: &[CommitData]) -> Vec<SearchResult> {
        // TODO: Filter multiple finds
        let search_str = "(cherry picked from commit ";
        commits
            .iter()
            .filter_map(|c| {
                if let Some(index) = c.message.find(search_str) {
                    let index = index + search_str.len();
                    if let Some(end_index) = c.message[index..].find(')') {
                        // we have to increase the end_index by the number of bytes that were cut off through slicing
                        let end_index = end_index + index;
                        let cherry_id = String::from(&c.message[index..end_index]);
                        return Some(SearchResult::new(
                            String::from(NAME),
                            [c.id.clone(), cherry_id],
                        ));
                    }
                }
                None
            })
            .collect::<Vec<SearchResult>>()
    }
}
