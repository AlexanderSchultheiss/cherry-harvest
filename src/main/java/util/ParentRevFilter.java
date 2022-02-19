package util;

import org.eclipse.jgit.errors.StopWalkException;
import org.eclipse.jgit.revwalk.RevCommit;
import org.eclipse.jgit.revwalk.RevWalk;
import org.eclipse.jgit.revwalk.filter.RevFilter;

import java.io.IOException;

/**
 * Filter for JGit commits that have at least min parents and at most max parents,
 * which can be used to identify merge commits.
 *
 * Not to be confused with CherryPick filters!
 *
 * @author Maike
 */

public class ParentRevFilter extends RevFilter {
    private final int min; // minimum number of parents allowed
    private final int max; // maximum number of parents allowed

    public ParentRevFilter(int min, int max){
        this.min = min;
        this.max = max;
    }
    @Override
    public boolean include(RevWalk revWalk, RevCommit revCommit) throws StopWalkException, IOException {
        return (revCommit.getParentCount() >= min && revCommit.getParentCount() <= max);
    }

    @Override
    public RevFilter clone() {
        return new ParentRevFilter(min, max);
    }
}
