use crate::method::SearchMethod;
use crate::git::CommitData;
use crate::CherryGroup;
use std::collections::HashMap;

#[derive(Default)]
pub struct MessageScan();

impl SearchMethod for MessageScan {
    fn search(&self, commits: &Vec<CommitData>) -> Vec<CherryGroup> {
        // TODO: Filter multiple finds
        let search_str = "(cherry picked from commit ";
        commits
            .iter()
            .filter_map(|c| {
                if let Some(index) = c.message.find(search_str) {
                    let index = index + search_str.len();
                    if let Some(end_index) = c.message[index..].find(")") {
                        // we have to increase the end_index by the number of bytes that were cut off through slicing
                        let end_index = end_index + index;
                        let cherry_id = String::from(&c.message[index..end_index]);
                        return Some(CherryGroup::new(vec![c.id.clone(), cherry_id]));
                    }
                }
                None
            })
            .collect::<Vec<CherryGroup>>()
    }
}
