package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;

import java.io.IOException;
import java.util.List;
import java.util.Set;

public interface CherrySearch {
    public Set<CherryPick> findAllCherryPicks() throws GitAPIException, IOException;
}
