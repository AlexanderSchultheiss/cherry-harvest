# Thoughts and Ideas

## Cherry-pick options
(source: https://git-scm.com/docs/git-cherry-pick)
### -e
* allows you to edit commit message
#### Thoughts
* also when using -x -> just search for pattern of 40-chars long alpha-numeric SHA1 - or do we assume that people would not touch this part of the commit message?
* potentially renders matchByMessageEquality useless

### --no-commit
* applies changes without committing, so leaving possibility to commit
* Usually the command automatically creates a sequence of commits. This flag applies the changes necessary to cherry-pick each named commit to your working tree and the index, without making any commit. In addition, when this option is used, your index does not have to match the HEAD commit. The cherry-pick is done against the beginning state of your index.
  * This is useful when cherry-picking more than one commits' effect to your index in a row.
#### Thoughts 
* so we also have to consider 1:n matchings i.t.o. cherry picked commits?
* makes it more difficult to identify cherry picks based on changes introduced
    
### --allow-empty
  * option allows empty commits to be preserved automatically in a cherry-pick
    Note that when "--ff" is in effect, empty commits that meet the "fast-forward" requirement will be kept even without this option. 
    Note also, that use of this option only keeps commits that were initially empty (i.e. the commit recorded the same tree as its parent). Commits which are made empty due to a previous commit are dropped. To force the inclusion of those commits use --keep-redundant-commits.

### --allow-empty-message
  * By default, cherry-picking a commit with an empty message will fail. This option overrides that behavior, allowing commits with empty messages to be cherry picked. 
#### Thoughts
  * if we base matching on patch ids, this should be irrelevant? 

### --keep-redundant-commits
  * If a commit being cherry picked duplicates a commit already in the current history, it will become empty. By default these redundant commits cause cherry-pick to stop so the user can examine the commit. This option overrides that behavior and creates an empty commit object. Implies --allow-empty. 
#### Thoughts
  * empty commit object -> not of interest for us, since it will be matched with the first (relevant) commit?

## Rebase vs. CherryPick
- Obvious indicator for cherry picks: commit message contains “cherry picked from”
- Could be used as a heuristic, but not really:
    - If we assume cherry-pick to be an action on individual commits -> chain of CherryPick pairs (i.e. where CherrySources s_1, ..., s_n build a chain and CherryTargets t_1,...,t_n build a chain, and (s_1,t_1),...,(s_n,t_n) build CherryPicks) corresponds to rebase 
      - Possible heuristics: threshold for time difference between commits, or just leverage parent relationship
      - BUT: we can also cherry-pick a range of commits which would threaten this approach.
    - rebase moves branch (pointer) to replayed commits, s.t. original commits should be unreachable
      - Possible heuristic: if the source commit is not reachable via any branch, that indicates rebase 
      - BUT: would that even happen when looking at a remote repo? Unreachable commits should not be uploaded to remote repo.
    - cherry-pick possibly used when we want to reuse commits from discontinued branches, that are messy and should not be merged, but want to use some commits from there
- Overall Problem: rebase seems to be a mostly local action that would not be retrievable from a public repository - except, commits are published and then rebased, which would not comply with golden rule (but could be d’accord with team’s agreement)
- if we have a branch that bases on another branch(, they merge at some point) and both share duplicate commits → could that indicate a rebase that broke the "golden rule"?
- check if complete branch has been rebased
    - 1) find split point (same grand^x parent) of two given commits (i.e. potential CherrySource and CherryTarget)
        - if none: assume cherry pick? "rebase -onto" could be a possible explanation as well though.
    - 2) check if all the commits from this split point to the head of each branch appear in cherry output as source and targets, respectively, in the same order
- check if CherrySource is on “inactive branch” (no commits since x days, no merging etc.) → dangling branch that was left over and could have possibly been rebased
    - but when does that happen?
        - rebased public branch (e.g. master) onto feature branch (not good practice?)
        - normally, you would rebase feature branch on master, then merge fast forward?
            - but if you rebase branch, publish it, then rebase it onto new head of master
                - if normal push: rejected, pull suggested, which merges it with “old” feature branch → duplicates (but only local at first, so one could clean that up)
                - if push —force : old commits deleted, only commits after rebase in repo
- Rebase gone wrong
    1. branch off main -> feature branch
    2. branch off feature branch → patch branch
    3. rebase feature branch -> patch branch bases on old commits of feature branch
    4. merge patch and feature branch → duplicate commits
    
    * Possible detection mechanism: check if they share split point and merge point → rebase gone wrong?


## Other things to think about
* until now, only 1:1 matches for cherry picks considered
    * think about whether this is always the case (excl. --no-commit) or there are exceptions
* How could we filter pairs of branches that do not need to be considered?

## TO DO
* Check if hash/equals for commit is ok?
* Check for same branch (compare it with itself)
* How to avoid duplicates while computing cherry picks?
* diff2listOfCommits
    * diff mehrfach
    * id eindeutig

    
# ChatGPT Proposal
To implement locality sensitive hashing (LSH) for text documents in Rust, you would first need to choose a similarity measure to use for comparing the documents. Some common similarity measures for text documents include the Jaccard similarity, which measures the overlap between the sets of words in the documents, and the cosine similarity, which measures the angle between the word vectors representing the documents.

Once you have chosen a similarity measure, you can implement LSH by following these steps:

    Preprocess the text documents to generate a set of words for each document. This can involve tokenizing the documents, removing stop words, stemming the words, and so on.

    Generate a hash function for each of the words in the vocabulary. This can involve choosing a random vector for each word, and then using the dot product between the vector and the word vector of each document as the hash value.

    For each document, compute the hash values for all the words in the document, and then combine the hash values using a suitable aggregation function, such as the min-hash or the SimHash.

    Use the combined hash value as the key for the document, and store the document in a hash table using the key as the index.

    To find the nearest neighbors of a query document, compute the hash value for the query document using the same steps as above, and then look up the hash table using the query's hash value as the key. This will return a list of documents that have similar hash values, which are likely to be similar to the query document.

You can use the Rust standard library to implement many of the steps above, such as the hash table and the hash functions. You may also want to use a Rust library for natural language processing, such as rust-nlp or rust-text-similarity, to help with the preprocessing and word vector generation steps.

# What is a good approach to preparing text for LSH?

There are several steps that are typically involved in preparing text for locality sensitive hashing (LSH). These steps can help to improve the performance of the LSH algorithm and the quality of the results it produces. Some of the most important steps are:

    Tokenization: This involves breaking the text into individual tokens, such as words, phrases, or symbols. This is typically done using a tokenizer, which is a specialized piece of software that can identify the tokens in the text and split the text into these tokens.

    Stop word removal: This involves removing common words from the text, such as "the", "and", and "but", which do not contribute much to the meaning of the text. These words are often called stop words, and they can be filtered out using a list of stop words.

    Stemming: This involves reducing words to their base form, or stem, in order to group together different inflections of the same word. For example, the stem of "running", "ran", and "run" would be "run". This can be done using a stemmer, which is a piece of software that can identify the stem of a word.

    Vectorization: This involves representing the text as a vector of numbers, where each dimension of the vector corresponds to a word in the vocabulary. The value of each dimension is determined by the frequency of the corresponding word in the text. This can be done using a vectorizer, which is a piece of software that can convert text into a numerical representation.

    Normalization: This involves scaling the values in the vector so that they have a consistent range and distribution. This can help to improve the performance of the LSH algorithm, as it can make the vectors more similar to each other and reduce the effect of outliers. This can be done using a normalizer, which is a piece of software that can apply a normalization function to the vector.

Overall, preparing text for LSH involves a number of different steps, which can help to improve the performance and accuracy of the LSH algorithm. These steps can be implemented using a combination of specialized software and custom code, depending on the specific requirements of the task at hand.

# What can I do if the syntax and grammar of the language at hand are unknown?

If the syntax and grammar of the language at hand are unknown, it can be difficult to analyze the text and extract useful information from it. In such cases, it may be necessary to use more general-purpose techniques that are not dependent on the specific syntax and grammar of the language.

One approach that can be taken is to use statistical methods that do not rely on the underlying structure of the language. For example, you could use term frequency-inverse document frequency (TF-IDF) to identify the most important words in the text, based on their frequency and rarity in the corpus. You could also use clustering algorithms, such as k-means or hierarchical clustering, to group the documents into clusters based on their similarity. These techniques can help to identify patterns and trends in the text, even if the syntax and grammar of the language are unknown.

Another approach is to use machine learning algorithms, such as deep learning models, which can learn to represent the text in a way that captures its meaning, even if the syntax and grammar are unknown. For example, you could use a recurrent neural network (RNN) or a convolutional neural network (CNN) to learn a vector representation of the text, where each dimension of the vector corresponds to a latent concept or feature of the text. This learned representation can then be used to measure the similarity between the texts, or to classify them into different categories, based on their content.

Overall, there are several approaches that can be taken if the syntax and grammar of the language at hand are unknown. These approaches can help to extract useful information from the text, even if the specific structure of the language is not known.

# HNSW vs. LSH
>What are hierarchical navigable small worlds?

 Hierarchical navigable small worlds, or HNSWs, are a type of data structure used to efficiently store and search through large datasets. They are often used in applications such as recommendation systems, where it is necessary to quickly find the closest matches to a given item in a large collection of data.

> Are they better than LSH?

 It depends on the specific use case. Locality-sensitive hashing (LSH) is a technique that is commonly used to approximate the nearest neighbors of a given item in a DATASET. HNSWs are a more recent development that can provide more accurate results for certain types of datasets and distance measures.

> For example?

As an example, consider a recommendation system for a streaming music service. In this case, the DATASET would consist of a large collection of songs, and the goal of the recommendation system would be to quickly find the songs that are most similar to a given song. LSH is a good technique for this problem when the similarity between songs is measured by their audio content. However, if the similarity is measured by other factors, such as the genre or artist of the songs, then HNSWs may be a better choice.

> So if the similarity should focus on the content rather than metadata, I should apply LSH?

Yes, that's correct. LSH is a good choice when the similarity between items is determined by their content, rather than by metadata such as labels or tags. This is because LSH is based on the idea of creating hash functions that preserve the similarity between items, and this works well when the similarity is determined by the content of the items.


# Which ANN indexing to choose?
![ann-choice.png](img%2Fann-choice.png)
(taken from [this website](https://towardsdatascience.com/comprehensive-guide-to-approximate-nearest-neighbors-algorith´ms-8b94f057d6b6))


# Notes on improvements - also regarding ngrammatic
I updated ngrammatic's implementation to remove a lot of the redundancy in instance management. 
The approach now scales in terms of memory consumption, but is still quite slow in terms of runtime.
ngrammatic's internal search heuristic is actually slower than brute force, because the search heuristic yields all elements (i.e., brute force) in the end


- split diffs into tokens (aka. alphanumeric words, can we split by !alphanumeric?)
- create a vocabulary of these tokens?
- use it for ANN?
- The lower the n in the n-gram approach, the faster the comparison
  - n=1 -> similarity based on frequency of certain characters
- Overall, a token-based approach should be faster
- Perhaps we should explicitly look for reverted commits?
  - our similarity approach would find reverted commits and classify them as cherry-pick, that is not what we intend
  - It could also be interesting to see, how often a commit was reverted, and then repeated again

# GitHub sampling 
The samplers should be concerned with sampling random Git repositories. ForkNetworks can then later be constructed from samples. 
This would have the benefit that we can sample a lot of repos once and then process them again and again. 
Also, we should offer the possibility to save a sample to file and load a sample from file.

Samplers should support retrieving a sample with a desired size and sampling repos on demand.


