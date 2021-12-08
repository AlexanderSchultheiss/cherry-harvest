import cherry.CherryPick;
import cherry.CherrySearch;
import cherry.GitCherrySearch;
import cherry.ScanCherrySearch;
import com.google.gson.Gson;
import org.eclipse.jgit.api.errors.GitAPIException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import util.Repository;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Path;
import java.time.LocalDateTime;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.TimeUnit;
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
            long start = System.nanoTime();
            Repository repository = new Repository(pathToGitRepository);
            cherrySearch = new ScanCherrySearch(repository);
            LOGGER.info("Starting cherry search.");
            final Set<CherryPick> cherrySet = cherrySearch.findAllCherryPicks();
            long end = System.nanoTime();
            long elapsed = TimeUnit.NANOSECONDS.toSeconds(end-start);
            LOGGER.info("Elapsed time: " + elapsed + "s");
            LOGGER.info("Finished cherry search.");

            if(cherrySet.isEmpty()){
                LOGGER.info("No cherry picks found!");
            } else {
                LOGGER.info("Number of unique cherry picks: " + cherrySet.size());

                Gson gson = new Gson();
                String[] pathSegments = pathToGitRepository.toString().split(Pattern.quote(File.separator));
                String pathName = "output/" + pathSegments[pathSegments.length - 1] +".json";
                LOGGER.info("Exporting cherry picks to " + pathName);
                FileWriter writer = new FileWriter(pathName);
                gson.toJson(cherrySet, writer);

                writer.flush();
                writer.close();
            }

            repository.close();
        } catch (IOException | GitAPIException e) {
            e.printStackTrace();
        }
    }
}
