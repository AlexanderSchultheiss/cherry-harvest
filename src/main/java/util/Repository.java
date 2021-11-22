package util;

import org.apache.commons.lang3.time.DateFormatUtils;
import org.apache.commons.lang3.time.DateUtils;
import org.eclipse.jgit.api.Git;
import org.eclipse.jgit.api.errors.GitAPIException;
import org.eclipse.jgit.errors.IncorrectObjectTypeException;
import org.eclipse.jgit.errors.MissingObjectException;
import org.eclipse.jgit.lib.ObjectId;
import org.eclipse.jgit.lib.Ref;
import org.eclipse.jgit.revwalk.RevCommit;
import org.eclipse.jgit.revwalk.RevWalk;

import java.io.IOException;
import java.nio.file.Path;
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
    private List<Ref> getLocalBranches() throws GitAPIException {
        return git.branchList().call();
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

    private Commit createCommit(String id, RevCommit rev){
        String message = rev.getFullMessage();
        Date time = new Date(seconds2milliseconds(rev.getCommitTime()));
        return new Commit(id, null, message, time);
    }

    private long seconds2milliseconds(int sec){
        return 1000 * (long) sec;
    }

}
