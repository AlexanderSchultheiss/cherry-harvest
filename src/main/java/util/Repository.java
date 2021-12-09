package util;

import org.eclipse.jgit.api.Git;
import org.eclipse.jgit.api.ListBranchCommand;
import org.eclipse.jgit.api.LogCommand;
import org.eclipse.jgit.api.errors.GitAPIException;
import org.eclipse.jgit.diff.DiffEntry;
import org.eclipse.jgit.diff.PatchIdDiffFormatter;
import org.eclipse.jgit.lib.ObjectId;
import org.eclipse.jgit.lib.ObjectReader;
import org.eclipse.jgit.lib.Ref;
import org.eclipse.jgit.revwalk.RevCommit;
import org.eclipse.jgit.revwalk.RevWalk;
import org.eclipse.jgit.revwalk.filter.RevFilter;
import org.eclipse.jgit.treewalk.CanonicalTreeParser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Path;
import java.util.*;
import java.util.concurrent.TimeUnit;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

public class Repository {
    final Logger LOGGER = LoggerFactory.getLogger(Repository.class);
    private final Path path;
    private Git git;

    public enum ListMode {
        LOCAL,
        REMOTE,
        ALL
    }


    public Repository(Path path) throws IOException {
        this.path = path;
        git = GitUtil.loadGitRepo(path.toFile());
    }


    public Path path() {
        return path;
    }


    /**
     * Uses id to find commit in the repository
     *
     * @param id Name of the commit
     * @return Handle for commit, representing commit in the repository
     */

    public Commit getCommitHandleById(String id, Branch branch) throws IOException {
        try (RevWalk walk = new RevWalk(git.getRepository())) {
            ObjectId objectId = ObjectId.fromString(id);
            RevCommit commit = walk.parseCommit(objectId);
            return createCommitHandle(commit, branch);
        } catch (IOException e) {
            throw e;
        }
    }


    /**
     * Computes patch id for diff between commit and its parent
     *
     * (Properties of patch id, see https://git-scm.com/docs/git-patch-id:
     * A patch id is "'reasonably stable', but at the same time also reasonably unique,
     * i.e., two patches that have the same 'patch ID' are almost guaranteed to be the same thing.
     * IOW, you can use this thing to look for likely duplicate commits.")
     *
     * @param commit Commit handle for which patch id for diff to parent is computed
     * @return Patch id for diff between given commit and its parent
     */
    public Optional<String> getPatchId(Commit commit) throws IOException, GitAPIException {
        // source: https://stackoverflow.com/questions/38664776/how-do-i-do-git-show-sha1-using-jgit
        ObjectId newTreeId = git.getRepository().resolve(commit.id() + "^{tree}");
        ObjectId oldTreeId = git.getRepository().resolve(commit.id() + "^^{tree}");

        if (newTreeId == null) {
            LOGGER.error("Could not resolve tree for commit with id " + commit.id());
            return Optional.empty();
        } else if (oldTreeId == null) {
            LOGGER.error("Could not resolve parent tree for commit with id " + commit.id());
            return Optional.empty();
        }

        try (ObjectReader reader = git.getRepository().newObjectReader()) {
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


    /**
     * Computes patch id for diff between commit and its parent
     *
     * (Properties of patch id, see https://git-scm.com/docs/git-patch-id:
     * A patch id is "'reasonably stable', but at the same time also reasonably unique,
     * i.e., two patches that have the same 'patch ID' are almost guaranteed to be the same thing.
     * IOW, you can use this thing to look for likely duplicate commits.")
     *
     * @param revCommit Commit for which patch id for diff to parent is computed
     * @return Patch id for diff between given commit and its parent
     */

    private Optional<String> getPatchId(RevCommit revCommit) throws IOException, GitAPIException {
        RevCommit parentCommit = revCommit.getParent(0);

        CanonicalTreeParser currentTreeParser = new CanonicalTreeParser();
        CanonicalTreeParser prevTreeParser = new CanonicalTreeParser();

        try (ObjectReader reader = git.getRepository().newObjectReader()) {
            if (revCommit.getTree() == null) {
                throw new RuntimeException("Could not obtain RevTree from child commit " + revCommit.getId());
            }
            if (parentCommit.getTree() == null) {
                throw new RuntimeException("Could not obtain RevTree from parent commit " + parentCommit.getId());
            }

            currentTreeParser.reset(reader, revCommit.getTree());
            prevTreeParser.reset(reader, parentCommit.getTree());
        }

        Optional<String> patchIdOptional = Optional.empty();

        try(PatchIdDiffFormatter formatter = new PatchIdDiffFormatter()) {
            formatter.setRepository(git.getRepository());
            formatter.format(currentTreeParser, prevTreeParser);

            String patchId = formatter.getCalulatedPatchId().getName();
            patchIdOptional = Optional.of(patchId);
        }

        return patchIdOptional;
    }


    public Set<Commit> getAllCommits() throws IOException, GitAPIException {
        return getCommits(Optional.empty());
    }


    public Set<Commit> getAllCommitsWithOneParent() throws IOException, GitAPIException {
        return getCommits(Optional.of(new ParentRevFilter(1,1)));
    }


    /**
     * Provides commit handles for a (sub-)set of the commits in the repository, depending on the filter
     *
     * @param revFilterOptional Predicate for filtering the commits
     * @return (Optionally filtered) Set of commit handles
     */

    private Set<Commit> getCommits(Optional<RevFilter> revFilterOptional) throws IOException, GitAPIException {
        Iterable <RevCommit> revCommits = getRevCommits(revFilterOptional);

        Set<Commit> commits = StreamSupport.stream(revCommits.spliterator(), false)
                .map(c -> createCommitHandle(c, null))
                .collect(Collectors.toSet());

        return commits;
    }

    /**
     * Retrieves commits from repository and filters them, if a predicate is given
     *
     * @param revFilterOptional Predicate for filtering the commits
     * @return (Optionally filtered) Set of commit handles
     */

    private Iterable<RevCommit> getRevCommits(Optional<RevFilter> revFilterOptional) throws GitAPIException, IOException {

        LogCommand log = git.log().all();

        if(revFilterOptional.isPresent()){
            RevFilter filter = revFilterOptional.get();
            log.setRevFilter(filter);
        }

        Iterable<RevCommit> revCommits = log.call();

        return revCommits;
    }

    /**
     * Computes cherry pick candidates based on patch ids.
     *
     * @return Commits that have the same patch id, organized as a map of patch id to commits
     */

    public Map<String, Set<Commit>> computeCherryPickCandidates() throws GitAPIException, IOException {
        Map<String, Set<Commit>> patch2commits = new HashMap<>();
        Iterable<RevCommit> revCommits = getRevCommits(Optional.of(new ParentRevFilter(1,1)));

        for(RevCommit rev: revCommits){
            Optional<String> patchOptional = this.getPatchId(rev);

            if(patchOptional.isPresent()){
                String patchID = patchOptional.get();
                if (patch2commits.containsKey(patchID)) {
                    patch2commits.get(patchID).add(createCommitHandle(rev, null));
                } else {
                    Set<Commit> similarCommits = new HashSet<>();
                    similarCommits.add(createCommitHandle(rev, null));
                    patch2commits.put(patchID, similarCommits);
                }
            }
        }

        return patch2commits;
    }


    /** Gets specific branches from repository, as specified by ListMode

        @param mode     Specifies which branches to get (local/remote/all)
        @return         List of branch handles
     */
    public List<Branch> getBranches(Repository.ListMode mode) throws GitAPIException {
        List<Branch> branches = new ArrayList<>();
        List<Ref> refList;

        switch (mode) {
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

        for (Ref b : refList) {
            branches.add(new Branch(b.getName()));
        }

        return branches;
    }


    /**
     * Creates Commit handle for given RevCommit (internal jgit representation of a commit)
     *
     * @param rev    RevCommit that is going to be represent by Commit handle
     * @param branch Branch which is considered in this case
     * @return
     */

    private Commit createCommitHandle(RevCommit rev, Branch branch) {
        String id = rev.getName();
        String message = rev.getFullMessage();
        Date time = new Date(TimeUnit.SECONDS.toMillis(rev.getCommitTime()));
        return new Commit(id, branch, message, time);
    }

    public void close() {
        git.close();
    }
}
