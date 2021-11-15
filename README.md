# cherry-search
A simple library for finding cherry-picks in git repositories. 

# Overview
With cherry-picking it is possible to apply the changes that happened in a previous commit to the current commit. The commit that is cherry-picked is called the cherry. The cherry is usually located in another branch (source branch) and contains changes that are required in the current branch (target branch). 

![](img/cherry-pick.png)

The goal of cherry-picking - as opposed to merging - is to only apply a subset of the changes that happened on the source branch. 
In git, cherry-picking can be done with the [command line call](https://git-scm.com/docs/git-cherry-pick):
```
git cherry-pick <commit>
```

For some of our research topics it might be interesting to analyze how cherry-picking is used in practice. Currently, we have a Bachelor's thesis that is looking into cherry-picking in practice, and that is facing an unresolved challenge: git does not track cherry-picks explicitly.
This means, that there is no built-in mechanism to find all cherry-picks in a project. 

The goal of this library is to offer the necessary functionality to address this challenge. 

# API
I have the following API in mind:
```java
// A record holding the commit id of a cherry that was picked
public record CherrySource(String commitId);

// A record holding the commit id of the commit that was created through cherry-picking. 
// This corresponds to the last commit in the `dev` branch of the example above
public record CherryTarget(String commitId);

// A record that represents a pair of cherry source and target
public record CherryPick(CherrySource source, CherryTarget target);

// Return all cherry picks that can be found in the given repository.
public static List<CherryPick> findAllCherryPicks(final Path pathToGitRepository);
```
which can then be called with:
```java
final Path pathToGitRepository = Path.of("/path/to/repo");
final List<CherryPick> cherryPicks = CherrySearch.findAllCherryPicks(pathToGitRepository);
```

# Possible Solution

## Git Cherry Command
Based on my exploration of a possible solution, I believe that the [git cherry](https://git-scm.com/docs/git-cherry) command might be the best place to start. As far as I understood it, it is possible to determine which commits in another branch have been applied to the current branch, and which have not. 

Challenges that I see:
- We cannot differentiate between `rebase` and `cherry-pick`. This is a problem which we cannot solve at the moment (I think). It is a threat to validity.
- I am unsure how the command treats merges. One should investigate whether already merged commits might be falsely interpreted as cherry-picks.
- The command does not consider all branches at once. Thus, we have to consider all possible pairwise combinations of branches in a repository to determine cherry-picks.

## Custom Commit Equality Check
If `git cherry` cannot be used as expected, we have to implement our own commit equality check. Two commits should be considered equal if the applied the same changes. Changes are identified by their content (i.e., the added or removed line) and their context (i.e., the lines that come directly before and after the change).