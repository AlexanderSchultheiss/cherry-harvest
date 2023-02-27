use crate::git::Commit;
use crate::search::methods::traditional_lsh::DiffSimilarity;
use crate::{CherryAndTarget, SearchMethod, SearchResult};
use faiss::{ConcurrentIndex, Index, LshIndex};
use firestorm::profile_method;
use log::info;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use std::collections::HashSet;
use std::time::Instant;

pub const NAME: &str = "RandomProjectionsLSH";

#[derive(Default)]
pub struct RandomProjectionsLSH {
    n_neighbors: usize,
    n_bits: u32,
    threshold: f64,
}

impl RandomProjectionsLSH {
    pub fn new(n_neighbors: usize, n_bits: u32, threshold: f64) -> Self {
        Self {
            n_neighbors,
            n_bits,
            threshold,
        }
    }
}

impl SearchMethod for RandomProjectionsLSH {
    fn search(&self, commits: &[Commit]) -> HashSet<SearchResult> {
        profile_method!("Faiss_search");
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
        let dim = embeddings[0].len();
        let mut index = LshIndex::new(dim as u32, self.n_bits).unwrap();
        for emb in &embeddings {
            index.add(emb).unwrap();
        }

        info!("finished table building in {:?}", start.elapsed());

        // Query in sublinear time.
        let mut cherries: HashSet<SearchResult> = HashSet::new();
        for (i, embedding) in embeddings.iter().enumerate() {
            let result = index.search(embedding, self.n_neighbors).unwrap();
            let mut similarity_comparator = DiffSimilarity::new();
            for neighbor_id in result.labels {
                let neighbor_id = match neighbor_id.get() {
                    None => continue,
                    Some(id) => id as usize,
                };
                let commit_a = &commits[i];
                let commit_b = &commits[neighbor_id];
                if commit_a.id() == commit_b.id() {
                    continue;
                }
                if similarity_comparator.change_similarity(commit_a, commit_b) > self.threshold {
                    let cherry_and_target = CherryAndTarget::construct(commit_a, commit_b);
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
