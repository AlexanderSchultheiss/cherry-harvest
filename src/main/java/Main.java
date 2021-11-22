import cherry.CherryPick;
import cherry.CherrySearch;
import util.Branch;
import util.UnequalCherryCandidatesException;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;

public class Main {
    public static void main(String... args) {
        final Path pathToGitRepository = Path.of("D:\\Maike\\git\\cherry");
        final CherrySearch cherrySearch;
        try {
            cherrySearch = new CherrySearch(pathToGitRepository);
            final List<CherryPick> cherryPicks = cherrySearch.findCherryPicks(new Branch("master"), new Branch("newb"));
            System.out.println(cherryPicks.toString());
        } catch (IOException | UnequalCherryCandidatesException e) {
            e.printStackTrace();
        }
    }
}
