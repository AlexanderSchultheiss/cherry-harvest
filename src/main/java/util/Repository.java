package util;

import org.apache.commons.lang3.NotImplementedException;
import org.apache.commons.lang3.time.DateFormatUtils;
import org.apache.commons.lang3.time.DateUtils;
import org.eclipse.jgit.api.Git;
import org.eclipse.jgit.api.errors.GitAPIException;
import org.eclipse.jgit.diff.DiffEntry;
import org.eclipse.jgit.diff.DiffFormatter;
import org.eclipse.jgit.diff.PatchIdDiffFormatter;
import org.eclipse.jgit.errors.IncorrectObjectTypeException;
import org.eclipse.jgit.errors.MissingObjectException;
import org.eclipse.jgit.lib.ObjectId;
import org.eclipse.jgit.lib.ObjectReader;
import org.eclipse.jgit.lib.Ref;
import org.eclipse.jgit.revwalk.RevCommit;
import org.eclipse.jgit.revwalk.RevWalk;
import org.eclipse.jgit.treewalk.CanonicalTreeParser;

import java.io.IOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;

public class Repository {
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

    public Commit getCommitById(String id) throws IOException {
        try( RevWalk walk = new RevWalk(git.getRepository()) ) {
            ObjectId objectId = ObjectId.fromString(id);
            RevCommit commit = walk.parseCommit(objectId);
            return createCommit(id, commit);
        } catch (IOException e) {
            e.printStackTrace();
            throw e;
        }
    }

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

    private Commit createCommit(String id, RevCommit rev){
        String message = rev.getFullMessage();
        Date time = new Date(seconds2milliseconds(rev.getCommitTime()));
        return new Commit(id, null, message, time);
    }

    private long seconds2milliseconds(int sec){
        return 1000 * (long) sec;
    }



}
