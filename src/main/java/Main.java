import cherry.*;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import filter.Filter;
import filter.MessageFilter;
import filter.OrFilter;
import filter.TimeFilter;
import org.eclipse.jgit.api.errors.GitAPIException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import util.Repository;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Path;
import java.util.Set;
import java.util.concurrent.TimeUnit;
import java.util.regex.Pattern;

/**
 * Entry point and example of how to use CherrySearch
 */
public class Main {
    final static Logger LOGGER = LoggerFactory.getLogger(Main.class);

    public static void main(String... args) {
        if(args.length == 0){
            LOGGER.error("No path to git directory given!");
            LOGGER.error("Aborting cherry search.");
            return;
        }

        final Path pathToGitRepository = Path.of(args[0]);

        if(!pathToGitRepository.toFile().exists()){
            LOGGER.error("Git repository does not exist at given path.");
            LOGGER.error("Aborting cherry search.");
            return;
        }

        final CherrySearch cherrySearch;

        try(Repository repository = new Repository(pathToGitRepository);) {
            long start = System.nanoTime();
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
                LOGGER.info("Number of cherry picks: " + cherrySet.size());

                Gson gson = new GsonBuilder()
                        .setDateFormat("EEE, dd MMM yyyy HH:mm:ss zzz").create();
                String[] pathSegments = pathToGitRepository.toString().split(Pattern.quote(File.separator));
                String pathName = "output/" + pathSegments[pathSegments.length - 1] +".json";
                LOGGER.info("Exporting cherry picks to " + pathName);
                FileWriter writer = new FileWriter(pathName);
                gson.toJson(cherrySet, writer);

                writer.flush();
                writer.close();


            }
        } catch (IOException | GitAPIException e) {
            LOGGER.error(e.getMessage());
            LOGGER.error("Aborting cherry search.");
        }
    }
}
