package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;

import java.io.IOException;
import java.util.List;
import java.util.Set;

public interface CherrySearch {
    /**
     * Searches for all cherry picks in a git repository.
     *
     * @return  Set of possible cherry picks
     */
    public Set<CherryPick> findAllCherryPicks() throws GitAPIException, IOException;
}
