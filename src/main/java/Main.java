import java.io.IOException;
import java.nio.file.Path;
import java.util.List;

public class Main {
    public static void main(String... args) {
        final Path pathToGitRepository = Path.of("/path/to/repo");
        final CherrySearch cherrySearch;
        try {
            cherrySearch = new CherrySearch(pathToGitRepository);
            final List<CherryPick> cherryPicks = cherrySearch.findAllCherryPicks(pathToGitRepository);
        } catch (IOException e) {
            e.printStackTrace();
        }
    }
}
