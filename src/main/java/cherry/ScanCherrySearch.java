package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import util.Commit;
import util.Repository;

import java.io.IOException;
import java.util.*;

public class ScanCherrySearch implements CherrySearch {
    final Logger LOGGER = LoggerFactory.getLogger(ScanCherrySearch.class);
    private Repository repository;

    public ScanCherrySearch(Repository repo) throws IOException {
        repository = repo;
    }

    @Override
    public Set<CherryPick> findAllCherryPicks() throws GitAPIException, IOException {
        Set<CherryPick> cherryPicks = new HashSet<>();
        Map<String, Set<Commit>> patchid2commits = new HashMap<>();

        LOGGER.info("Fetching all eligible commits from repository.");
        Set<Commit> commits = repository.getAllCommitsWithOneParent();

        LOGGER.info("Computing patch ids.");
        for (Commit c : commits) {
            Optional<String> patchOptional = repository.getPatchId(c);

            if(patchOptional.isPresent()){
                String patchID = patchOptional.get();
                if (patchid2commits.containsKey(patchID)) {
                    patchid2commits.get(patchID).add(c);
                } else {
                    Set<Commit> similarCommits = new HashSet<>();
                    similarCommits.add(c);
                    patchid2commits.put(patchID, similarCommits);
                }
            }
        }

        LOGGER.info("Computing cherry picks.");
        for (Set<Commit> commitSet : patchid2commits.values()) {
            if (commitSet.size() > 1) {
                final List<Commit> commitList = new ArrayList<Commit>(commitSet);

                for (int i = 0; i < commitList.size() - 1; ++i) {
                    for (int j = i+1; j < commitList.size(); ++j){
                        cherryPicks.add(CherryPick.determineSourceAndTarget(commitList.get(i), commitList.get(j)));
                    }
                }
            }
        }

        return cherryPicks;
    }
}