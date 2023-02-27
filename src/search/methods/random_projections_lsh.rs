use crate::git::{Commit, Diff};
use crate::search::methods::traditional_lsh::preprocessing::preprocess_commits;
use crate::search::methods::traditional_lsh::DiffSimilarity;
use crate::{CherryAndTarget, SearchMethod, SearchResult};
use faiss::LshIndex;
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
        info!("finished diff embedding in {:?}", start.elapsed());

        let output = model.encode(&diffs);
        let embeddings = output.unwrap();

        // Do one time expensive preprocessing.
        let start = Instant::now();
        use faiss::{index_factory, Index, MetricType};
        let dim = diffs[0].len();
        let mut index = LshIndex::new(dim as u32, 24).unwrap();
        for emb in embeddings {
            index.add(&emb).unwrap();
        }

        info!("finished table building in {:?}", start.elapsed());

        // Query in sublinear time.
        // let result = lsh.query_bucket_ids(embeddings.get(0).unwrap());
        // let result = result.unwrap();
        // println!("result: {result:?}");
        //
        // let mut similarity_comparator = DiffSimilarity::new();
        // println!("query: {}", commits.get(0).unwrap().message());
        // for r in result {
        //     println!("result {r}: {}", commits.get(r as usize).unwrap().message());
        //     let commit_a = &commits[0];
        //     let commit_b = &commits[r as usize];
        //     if commit_a.id() == commit_b.id() {
        //         continue;
        //     }
        //     if similarity_comparator.change_similarity(commit_a, commit_b) > 0.75 {
        //         let cherry_and_target = CherryAndTarget::construct(commit_a, commit_b);
        //         println!("CaT: {cherry_and_target:?}")
        //     }
        // }
        todo!();
    }

    fn name(&self) -> &'static str {
        NAME
    }
}
