package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import util.Commit;
import util.Repository;

import java.io.IOException;
import java.util.*;

/**
 * ScanCherrySearch enables search for cherry picks by scanning all the commits in a repository,
 * looking for matching commits based on patch id.
 *
 * @author Maike
 */

public class ScanCherrySearch implements CherrySearch {
    final Logger LOGGER = LoggerFactory.getLogger(ScanCherrySearch.class);
    private Repository repository;

    public ScanCherrySearch(Repository repo) throws IOException {
        this.repository = repo;
    }

    @Override
    public Set<CherryPick> findAllCherryPicks() throws GitAPIException, IOException {
        Set<CherryPick> cherryPicks = new HashSet<>();

        LOGGER.info("Fetching all eligible commits from repository and computing fetch ids.");
        Map<String, Set<Commit>> patchid2commits = repository.computeCherryPickCandidates();

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

    /**
     * Matches all commits in set pairwise (except to themselves),
     * as all have the same patch id.
     *
     * @param commitSet Set of commits having the same patch id
     * @return Set of possible CherryPicks
     */
    public Set<CherryPick> matchAll(Set<Commit> commitSet){
        final List<Commit> commitList = new ArrayList<Commit>(commitSet);
        Set<CherryPick> cherryPicks = new HashSet<>();

        for (int i = 0; i < commitList.size() - 1; ++i) {
            for (int j = i+1; j < commitList.size(); ++j){
                cherryPicks.add(CherryPick.determineSourceAndTarget(commitList.get(i), commitList.get(j)));
            }
        }

        return cherryPicks;
    }

    /**
     * Matches all commits in the set to the oldest commit (except to itself),
     * as it is assumed to be the "original" commit.
     * The oldest commit serves as the CherrySource in all CherryPicks,
     * the other commits are the CherryTarget, respectively.
     *
     * @param commitSet Set of commits having the same patch id
     * @return Set of possible CherryPicks
     */
    public Set<CherryPick> matchOldestWithRest(Set<Commit> commitSet){
        Set<CherryPick> cherryPicks = new HashSet<>();

        if(commitSet.size() > 1){
            // TODO: new CherrySource object for each CherryPick, or reused in every CherryPick?
            // (should not make a difference due to equals method in commit?)
            CherrySource source = findSourceCommit(commitSet);

            for(Commit c: commitSet){
                if(!c.equals(source.commit())){
                    cherryPicks.add(new CherryPick(source, new CherryTarget(c)));
                }
            }
        }

        return cherryPicks;
    }

    /**
     * Picks a commit as the CherrySource from a given set of commits.
     *
     * @param commitSet Set of commits having the same patch id
     * @return Oldest commit as CherrySource
     */
    private CherrySource findSourceCommit(Set<Commit> commitSet) {
        Commit oldest = Collections.min(commitSet, Comparator.comparing(Commit::timestamp));
        CherrySource source = new CherrySource(oldest);
        return source;
    }
}