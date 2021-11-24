import cherry.CherryPick;
import cherry.CherrySearch;
import cherry.CherrySource;
import com.google.gson.Gson;
import org.eclipse.jgit.api.errors.GitAPIException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import util.Branch;
import util.Repository;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import java.util.regex.Pattern;

public class Main {
    final static Logger LOGGER = LoggerFactory.getLogger(Main.class);

    public static void main(String... args) {
        if(args == null){
            throw new RuntimeException("No path to git directory given!");
        }

        final Path pathToGitRepository = Path.of(args[0]);
        final CherrySearch cherrySearch;

        try {
            Repository repository = new Repository(pathToGitRepository);
            cherrySearch = new CherrySearch(repository);
            LOGGER.info("Starting cherry search.");
            final List<CherryPick> cherryPicks = cherrySearch.findAllCherryPicks();

            LOGGER.info("_______________________________________");
            LOGGER.info("Finished cherry search.");

            if(cherryPicks.isEmpty()){
                LOGGER.info("No cherry picks found!");
            } else {
                LOGGER.info("Size: " + cherryPicks.size());
            }

            Gson gson = new Gson();
            String[] pathSegments = pathToGitRepository.toString().split(Pattern.quote(File.separator));
            LOGGER.info(pathSegments[pathSegments.length-1]);
            FileWriter writer = new FileWriter("output/" + pathSegments[pathSegments.length - 1] +".json");
            gson.toJson(cherryPicks, writer);

            writer.flush();
            writer.close();
            repository.close();
        } catch (IOException | GitAPIException e) {
            e.printStackTrace();
        }
    }
}
