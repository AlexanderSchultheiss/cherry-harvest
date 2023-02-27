use crate::git::{Commit, Diff};
use crate::search::methods::traditional_lsh::preprocessing::preprocess_commits;
use crate::search::methods::traditional_lsh::DiffSimilarity;
use crate::{CherryAndTarget, SearchMethod, SearchResult};
use faiss::{Index, LshIndex};
use firestorm::{profile_fn, profile_method};
use log::{debug, info};
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub const NAME: &str = "RandomProjectionsLSH";

#[derive(Default)]
pub struct RandomProjectionsLSH();

impl RandomProjectionsLSH {
    pub fn new() -> Self {
        Self()
    }
}

impl SearchMethod for RandomProjectionsLSH {
    fn search(&self, commits: &[Commit]) -> HashSet<SearchResult> {
        info!("searching with random projections");
        let start = Instant::now();
        let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
            .create_model()
            .unwrap();
        let diffs: Vec<&str> = commits.iter().map(|c| c.diff().diff_text()).collect();

        let output = model.encode(&diffs);
        let embeddings = output.unwrap();

        info!("finished diff embedding in {:?}", start.elapsed());

        // Do one time expensive preprocessing.
        let start = Instant::now();
        use faiss::{index_factory, Index, MetricType};
        let dim = diffs[0].len();
        let mut index = LshIndex::new(dim as u32, 24).unwrap();
        for emb in &embeddings {
            index.add(emb).unwrap();
        }

        info!("finished table building in {:?}", start.elapsed());

        // Query in sublinear time.
        let n_neighbors = 10;
        let mut cherries: HashSet<SearchResult> = HashSet::new();
        debug!("embeddings_size: {}", embeddings.len());
        for (i, embedding) in embeddings.iter().enumerate() {
            let result = index.search(embedding, n_neighbors).unwrap();
            let mut similarity_comparator = DiffSimilarity::new();
            // println!("query: {}", commits.get(0).unwrap().message());
            for neighbor_id in result.labels {
                let neighbor_id = neighbor_id.get().unwrap() as usize;
                debug!(
                    "result {neighbor_id}: {}",
                    commits.get(neighbor_id).unwrap().message()
                );
                let commit_a = &commits[i];
                let commit_b = &commits[neighbor_id];
                if commit_a.id() == commit_b.id() {
                    continue;
                }
                if similarity_comparator.change_similarity(commit_a, commit_b) > 0.75 {
                    let cherry_and_target = CherryAndTarget::construct(commit_a, commit_b);
                    debug!("CaT: {cherry_and_target:?}");
                    cherries.insert(SearchResult {
                        search_method: self.name().to_string(),
                        cherry_and_target,
                    });
                }
            }
        }
        cherries
    }

    fn name(&self) -> &'static str {
        NAME
    }
}
