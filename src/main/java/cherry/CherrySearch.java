package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;

import java.io.IOException;
import java.util.List;

public interface CherrySearch {
    public List<CherryPick> findAllCherryPicks() throws GitAPIException, IOException;
}
