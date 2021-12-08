package util;

import org.eclipse.jgit.api.Git;
import org.eclipse.jgit.api.ListBranchCommand;
import org.eclipse.jgit.api.errors.GitAPIException;
import org.eclipse.jgit.diff.DiffEntry;
import org.eclipse.jgit.diff.PatchIdDiffFormatter;
import org.eclipse.jgit.lib.ObjectId;
import org.eclipse.jgit.lib.ObjectReader;
import org.eclipse.jgit.lib.Ref;
import org.eclipse.jgit.revwalk.RevCommit;
import org.eclipse.jgit.revwalk.RevWalk;
import org.eclipse.jgit.treewalk.CanonicalTreeParser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Path;
import java.util.*;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

public class Repository {
    final Logger LOGGER = LoggerFactory.getLogger(Repository.class);
    private final Path path;
    private Git git;

    public enum ListMode{
        LOCAL,
        REMOTE,
        ALL
    }

    public Repository(Path path) throws IOException {
        this.path = path;
        git = GitUtil.loadGitRepo(path.toFile());
    }

    public Path path(){
        return path;
    }

    /**
     * Uses id to find commit in the repository
     * @param id Name of the commit
     * @return  Handle for commit, representing commit in the repository
     */

    public Commit getCommitHandleById(String id, Branch branch) throws IOException {
        try( RevWalk walk = new RevWalk(git.getRepository())) {
            ObjectId objectId = ObjectId.fromString(id);
            RevCommit commit = walk.parseCommit(objectId);
            return createCommitHandle(commit, branch);
        } catch (IOException e) {
            throw e;
        }
    }

    /**
        Computes diff between commit and its parent,
        which is then used to generate patch id
        @param commit   Commit for which diff to parent (and patch id) is computed
        @return         Patch id for diff between given commit and its parent
     */
    public Optional<String> getPatchId(Commit commit) throws IOException, GitAPIException {
        // source: https://stackoverflow.com/questions/38664776/how-do-i-do-git-show-sha1-using-jgit
        ObjectId newTreeId = git.getRepository().resolve(commit.id() + "^{tree}");
        ObjectId oldTreeId = git.getRepository().resolve(commit.id() + "^^{tree}");

        if(newTreeId == null){
            LOGGER.error("Could not resolve tree for commit with id " + commit.id());
            return Optional.empty();
        } else if (oldTreeId == null){
            LOGGER.error("Could not resolve parent tree for commit with id " + commit.id());
            return Optional.empty();
        }

        try(ObjectReader reader = git.getRepository().newObjectReader() ){
            CanonicalTreeParser newTree = new CanonicalTreeParser();
            newTree.reset(reader, newTreeId);

            CanonicalTreeParser oldTree = new CanonicalTreeParser();
            oldTree.reset(reader, oldTreeId);

            List<DiffEntry> diffEntries = git.diff().setNewTree(newTree).setOldTree(oldTree).call();
            PatchIdDiffFormatter formatter = new PatchIdDiffFormatter();
            formatter.setRepository(git.getRepository());
            formatter.format(diffEntries);

            String patchId = formatter.getCalulatedPatchId().getName();

            formatter.close();
            return Optional.of(patchId);
        } catch (IOException | GitAPIException e) {
            throw e;
        }
    }

    public Set<Commit> getAllCommits() throws IOException, GitAPIException {
        return getCommits(Optional.empty());
    }

    public Set<Commit> getAllCommitsWithOneParent() throws IOException, GitAPIException {
        return getCommits(Optional.of((c -> c.getParentCount() == 1)));
    }

    /**
     * Retrieves commits from repository and filters them, if a predicate is given
     *
     * @param predicateOptional     Predicate for filtering the commits
     * @return  (Optionally filtered) Set of commit handles
     */
    private Set<Commit> getCommits(Optional<Predicate<RevCommit>> predicateOptional) throws IOException, GitAPIException {
        Set<Commit> commits;
        Iterable<RevCommit> revCommits = git.log().all().call();
        Predicate<RevCommit> predicate = predicateOptional.isEmpty()? (c -> true) : predicateOptional.get();

        commits = StreamSupport.stream(revCommits.spliterator(), false)
                .filter(predicate)
                .map(c -> createCommitHandle(c, null))
                .collect(Collectors.toSet());

        return commits;
    }

    public List<Branch> getBranches(Repository.ListMode mode) throws GitAPIException {
        List<Branch> branches = new ArrayList<>();
        List<Ref> refList;

        switch (mode){
            case ALL:
                refList = git.branchList().setListMode(ListBranchCommand.ListMode.ALL).call();
                break;
            case REMOTE:
                refList = git.branchList().setListMode(ListBranchCommand.ListMode.REMOTE).call();
                break;
            case LOCAL:
                refList = git.branchList().call();
                break;
            default:
                LOGGER.warn("List Mode not known - will only get local branches.");
                refList = git.branchList().call();
        }

        for(Ref b : refList){
            branches.add(new Branch(b.getName()));
        }

        return branches;
    }

    private Commit createCommitHandle(RevCommit rev, Branch branch){
        String id = rev.getName();
        String message = rev.getFullMessage();
        Date time = new Date(seconds2milliseconds(rev.getCommitTime()));
        return new Commit(id, branch, message, time);
    }

    private long seconds2milliseconds(int sec){
        return 1000 * (long) sec;
    }

    public void close(){
        git.close();
    }
}
