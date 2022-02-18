import cherry.CherryPick;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;
import filter.Filter;
import filter.MessageFilter;
import filter.OrFilter;
import filter.TimeFilter;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.*;
import java.nio.file.Path;

import java.util.Set;

public class ResultsFilter {
    final static Logger LOGGER = LoggerFactory.getLogger(ResultsFilter.class);

    public static void main(String... args) {
        if(args.length == 0){
            LOGGER.error("No path to file given!");
            LOGGER.error("Aborting filtering step.");
            return;
        }

        final Path path = Path.of(args[0]);

        if(!path.toFile().exists()){
            LOGGER.error("File does not exist at given path.");
            LOGGER.error("Aborting filtering step.");
            return;
        }

        try {
            ObjectMapper objectMapper = new ObjectMapper();
            Set<CherryPick> cherryPicks = objectMapper.readValue(path.toFile(), new TypeReference<Set<CherryPick>>(){});

            Filter filter = new OrFilter(new MessageFilter(), new TimeFilter());
            Set<CherryPick> filtered = filter.filter(cherryPicks);

            LOGGER.info("Number of filtered cherry picks: " + filtered.size());

            Gson gson = new GsonBuilder()
                    .setDateFormat("EEE, dd MMM yyyy HH:mm:ss zzz").create();
            String pathNameFiltered = path.toString().replace(".json", "_filtered.json");
            LOGGER.info("Exporting filtered cherry picks to " + pathNameFiltered);
            FileWriter writer = new FileWriter(pathNameFiltered);
            gson.toJson(filtered, writer);

            writer.flush();
            writer.close();
        } catch (IOException e) {
            e.printStackTrace();
        }

    }

}

