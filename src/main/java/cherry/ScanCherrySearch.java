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
        patchid2commits = repository.computeCherryPickCandidates();

        LOGGER.info("Computing cherry picks.");
        for (Set<Commit> commitSet : patchid2commits.values()) {
            if (commitSet.size() > 2) {
                cherryPicks.addAll(matchOldestWithRest(commitSet));
            } else if (commitSet.size() == 2){
                Commit[] c = new Commit[2];
                commitSet.toArray(c);
                cherryPicks.add(CherryPick.determineSourceAndTarget(c[0], c[1]));
            }
        }

        return cherryPicks;
    }

    private CherrySource findSourceCommit(Set<Commit> commitSet) {
        Commit oldest = Collections.min(commitSet, Comparator.comparing(Commit::timestamp));
        CherrySource source = new CherrySource(oldest);
        return source;
    }

    private Set<CherryPick> matchAll(Set<Commit> commitSet){
        final List<Commit> commitList = new ArrayList<Commit>(commitSet);
        Set<CherryPick> cherryPicks = new HashSet<>();

        for (int i = 0; i < commitList.size() - 1; ++i) {
            for (int j = i+1; j < commitList.size(); ++j){
                cherryPicks.add(CherryPick.determineSourceAndTarget(commitList.get(i), commitList.get(j)));
            }
        }

        return cherryPicks;
    }

    private Set<CherryPick> matchOldestWithRest(Set<Commit> commitSet){
        CherrySource source = findSourceCommit(commitSet);
        Set<CherryPick> cherryPicks = new HashSet<>();

        for(Commit c: commitSet){
            if(c != source.commit()){
                cherryPicks.add(new CherryPick(source, new CherryTarget(c)));
            }
        }

        return cherryPicks;
    }
}