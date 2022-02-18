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