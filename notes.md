#Thoughts and Ideas

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

## TODO
* investigate merges and rebase in the context of cherry 
* until now, only 1:1 matches for cherry picks considered
    * think about whether this is always the case (excl. --no-commit) or there are exceptions
* How to avoid duplicates while computing cherry picks?
* How could we filter pairs of branches that do not need to be considered?
* Check if hash/equals for commit is ok?
  
* Check for same branch (compare it with itself)
* Commits 
* diff2listOfCommits
    * diff mehrfach
    * id eindeutig