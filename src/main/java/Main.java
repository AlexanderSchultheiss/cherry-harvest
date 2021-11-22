import cherry.CherryPick;
import cherry.CherrySearch;
import org.eclipse.jgit.api.errors.GitAPIException;
import util.Branch;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;

public class Main {
    public static void main(String... args) {
        if(args == null){
            throw new RuntimeException("No path to git directory given!");
        }
        final Path pathToGitRepository = Path.of(args[0]);
        final CherrySearch cherrySearch;
        try {
            cherrySearch = new CherrySearch(pathToGitRepository);
            final List<CherryPick> cherryPicks = cherrySearch.findCherryPicks(new Branch("master"), new Branch("newb"));
            System.out.println(cherryPicks.toString());
        } catch (IOException | GitAPIException e) {
            e.printStackTrace();
        }
    }
}
