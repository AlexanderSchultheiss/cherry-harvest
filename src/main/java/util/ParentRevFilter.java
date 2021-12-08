package util;

import org.eclipse.jgit.errors.IncorrectObjectTypeException;
import org.eclipse.jgit.errors.MissingObjectException;
import org.eclipse.jgit.errors.StopWalkException;
import org.eclipse.jgit.revwalk.RevCommit;
import org.eclipse.jgit.revwalk.RevWalk;
import org.eclipse.jgit.revwalk.filter.RevFilter;

import java.io.IOException;

/**
    Filter for commits that have at least min parents and at most max parents.
 */

public class ParentRevFilter extends RevFilter {
    private final int min;
    private final int max;

    public ParentRevFilter(int min, int max){
        this.min = min;
        this.max = max;
    }
    @Override
    public boolean include(RevWalk revWalk, RevCommit revCommit) throws StopWalkException, MissingObjectException, IncorrectObjectTypeException, IOException {
        return (revCommit.getParentCount() >= min && revCommit.getParentCount() <= max);
    }

    @Override
    public RevFilter clone() {
        return new ParentRevFilter(min, max);
    }
}
