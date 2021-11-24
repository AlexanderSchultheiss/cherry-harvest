package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import util.Branch;
import util.Repository;

import java.io.IOException;
import java.nio.file.Path;

public class CherrySearchTest {
    static Repository repository;
    static CherrySearch cherrySearch;
    final static Path pathToGitRepository = Path.of("D:\\Maike\\git\\superset");
    // refs/heads/0.26 refs/heads/fix_typo
    //  refs/heads/fix_tablecolumn refs/heads/0.26

    @BeforeAll
    public static void setup() throws IOException {
        repository = new Repository(pathToGitRepository);
        cherrySearch = new CherrySearch(repository);
    }

    @Test
    public void testFindCherryPick() throws GitAPIException, IOException {
        cherrySearch.findCherryPicks(new Branch("refs/heads/0.26"), new Branch("refs/heads/fix_tablecolumn"));
    }

    @AfterAll
    public static void teardown(){
        repository.close();
    }

}
