package util;

import org.eclipse.jgit.api.CreateBranchCommand;
import org.eclipse.jgit.api.Git;
import org.eclipse.jgit.api.ListBranchCommand;
import org.eclipse.jgit.api.errors.GitAPIException;
import org.eclipse.jgit.diff.DiffEntry;
import org.eclipse.jgit.diff.PatchIdDiffFormatter;
import org.eclipse.jgit.lib.ObjectId;
import org.eclipse.jgit.lib.ObjectReader;
import org.eclipse.jgit.lib.Ref;
import org.eclipse.jgit.lib.SymbolicRef;
import org.eclipse.jgit.revwalk.RevCommit;
import org.eclipse.jgit.revwalk.RevWalk;
import org.eclipse.jgit.treewalk.CanonicalTreeParser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.IOException;
import java.nio.file.Path;
import java.util.*;

public class Repository {
    final Logger LOGGER = LoggerFactory.getLogger(Repository.class);
    private final Path path;
    private Git git;

    public Repository(Path path) throws IOException {
        this.path = path;
        git = GitUtil.loadGitRepo(path.toFile());
    }

    public Path path(){
        return path;
    }

    /**
     * Lists local branches available in VariantsRepository (git branch)
     */
    public List<Branch> getLocalBranches() throws GitAPIException {
        List<Ref> branchList = git.branchList().call();
        List<Branch> branches = new ArrayList<>();

        for(Ref b : branchList){
            branches.add(new Branch(b.getName()));
        }

        return branches;
    }

    /**
     * Uses id to find commit in the repository
     * @param id Name of the commit
     * @return  Handle for commit, representing commit in the repository
     * @throws IOException
     */

    public Commit getCommitHandleById(String id, Branch branch) throws IOException {
        try( RevWalk walk = new RevWalk(git.getRepository())) {
            ObjectId objectId = ObjectId.fromString(id);
            RevCommit commit = walk.parseCommit(objectId);
            return createCommitHandle(id, commit, branch);
        } catch (IOException e) {
            throw e;
        }
    }

    /**
        Computes diff between commit and its parent,
        which is then used to generate patch id
     */
    public String getPatchId(Commit commit) throws IOException, GitAPIException {
        // source: https://stackoverflow.com/questions/38664776/how-do-i-do-git-show-sha1-using-jgit
        ObjectId newTreeId = git.getRepository().resolve(commit.id() + "^{tree}");
        ObjectId oldTreeId = git.getRepository().resolve(commit.id() + "^^{tree}");

        try( ObjectReader reader = git.getRepository().newObjectReader() ){
            CanonicalTreeParser newTree = new CanonicalTreeParser();
            newTree.reset(reader, newTreeId);

            CanonicalTreeParser oldTree = new CanonicalTreeParser();
            oldTree.reset(reader, oldTreeId);

            List<DiffEntry> diffEntries = git.diff().setNewTree(newTree).setOldTree(oldTree).call();
            PatchIdDiffFormatter formatter = new PatchIdDiffFormatter();
            formatter.setRepository(git.getRepository());
            formatter.format(diffEntries);

            String patchId = formatter.getCalulatedPatchId().toString();

            formatter.close();
            return patchId;
        } catch (IOException | GitAPIException e) {
            throw e;
        }
    }

    public void checkoutAllBranches() throws GitAPIException, IOException {
        List<Ref> branchList = git.branchList().setListMode(ListBranchCommand.ListMode.ALL).call();
        Collection<Ref> branches = filterNonLocalBranches(branchList);
        for(Ref b : branches){
            String refName = b.getName();
            if(refName.startsWith("refs/remotes/") && !(b instanceof SymbolicRef)){
                String branchName = b.getName().replace("refs/remotes/origin/", "");
                git.checkout().
                        setCreateBranch(true).
                        setName(branchName).
                        setUpstreamMode(CreateBranchCommand.SetupUpstreamMode.TRACK).
                        setStartPoint("origin/" + branchName).
                        call();
            }
        }
    }

    private Collection<Ref> filterNonLocalBranches(List<Ref> branches) {
        Map<String, Ref> name2ref = new HashMap<>();

        for(Ref b : branches){
            if(!(b instanceof SymbolicRef)) {
                String name = b.getName();
                String shortName;
                if (name.startsWith("refs/remotes/")) {
                    shortName = name.replace("refs/remotes/origin/", "");
                } else if (name.startsWith("refs/heads/")) {
                    shortName = b.getName().replace("refs/heads/", "");
                } else {
                    LOGGER.debug("Cannot process branch " + name);
                    continue;
                }

                if (name2ref.containsKey(shortName)) {
                    name2ref.remove(shortName);
                } else {
                    name2ref.put(shortName, b);
                }
            }
        }

        return name2ref.values();
    }

    private Commit createCommitHandle(String id, RevCommit rev, Branch branch){
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
