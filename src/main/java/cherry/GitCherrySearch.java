package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import util.*;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.util.*;

/**
 * GitCherrySearch enables search for cherry picks based on <git cherry> command.
 * GitCherrySearch is kind of deprecated - using ScanCherrySearch is recommended instead, as it is much faster.
 *
 * @author Maike
 */

@Deprecated
public class GitCherrySearch implements CherrySearch {
    final Logger LOGGER = LoggerFactory.getLogger(GitCherrySearch.class);
    private Repository repository;

    public GitCherrySearch(Repository repo) throws IOException {
        repository = repo;
    }

    /**
        Finds all cherry picks in the given repository
        by computing cherry picks for all relevant pairs of branches
     */
    public Set<CherryPick> findAllCherryPicks() throws GitAPIException, IOException {
        final ArrayList<Branch> branches = new ArrayList<Branch>(repository.getBranches(Repository.ListMode.REMOTE));
        Set<CherryPick> cherryPicks = new HashSet<>();

        for(int i = 0; i < branches.size()-1 ; ++i){
            Branch branch1 =  branches.get(i);

            for(Branch branch2 : branches.subList(i+1, branches.size())){
                List<CherryPick> cherries = findCherryPicks(branch1, branch2);
                cherryPicks.addAll(cherries);
            }
        }

        return cherryPicks;
    }

    /**
     * Finds cherry picks in two given (distinct) branches
     *
     * @param branch1 that may hold commits representing cherry sources or targets
     * @param branch2      Branch that may hold commits representing cherry sources or targets
     * @return          Cherry picks that were identified
     * @throws IOException
     * @throws GitAPIException
     */
    public List<CherryPick> findCherryPicks(Branch branch1, Branch branch2) throws IOException, GitAPIException {
        // see https://stackoverflow.com/questions/2922652/git-is-there-a-way-to-figure-out-where-a-commit-was-cherry-picked-from :
        // call <git cherry upstream head> -> record '-' output
        // call <git cherry head upstream> -> record '-' output
        // need to match those -> either try to find commit messages including id of original commit (when -x option used)
        // or use patch id / commit messages to match candidates
        // when matched, determine which is source and which target by checking time stamps

        final List<String> branch1AsHead = cherry(branch2, branch1);
        final List<String> branch2AsHead = cherry(branch1, branch2);

        Map<String, Commit> commitsBranch1 = getCommits(branch1AsHead, branch1);
        Map<String, Commit> commitsBranch2 = getCommits(branch2AsHead, branch2);

        List<CherryPick> cherries = computeCherryPicks(commitsBranch2, commitsBranch1);

        if(commitsBranch1.size() != 0){
            LOGGER.info("Non-matched commits from branch 1: "+ commitsBranch1.values());
        }

        if(commitsBranch2.size() != 0){
            LOGGER.info("Non-matched commits from branch 2: "+ commitsBranch2.values());
        }

        return cherries;
    }

    /**
        "Wrapper" for shell command <git cherry upstream head>

        @return Relevant output that has been parsed to be used in this context
     */
    private List<String> cherry(Branch upstream, Branch head) throws IOException {
        final String[] command = {"git", "cherry", upstream.name(), head.name()};
        return parseOutput(executeCommand(command));
    }

    /**
     * Computes CherryPicks by using strategies depending on the size of the input.
     *
     * @param output1   Set of commits eligible for cherry picks from one direction (i.e. git cherry branch1 branch2)
     * @param output2   Set of commits eligible for cherry picks from other direction (git cherry branch2 branch1)
     * @return  Set of CherryPicks with commits for which a match was found
     * @throws GitAPIException
     * @throws IOException
     */

    private List<CherryPick> computeCherryPicks(Map<String,Commit> output1, Map<String, Commit> output2) throws GitAPIException, IOException {
        List<CherryPick> cherryPicks = new ArrayList<>();

        if(output1.size() == 1 && output2.size() == 1){
            // immediate match, just need to figure out source and target
            Commit commit1 = output1.values().iterator().next();
            Commit commit2 = output2.values().iterator().next() ;
            cherryPicks.add(CherryPick.determineSourceAndTarget(commit1, commit2));
            output1.clear();
            output2.clear();
        } else if(output1.size() == 0 || output2.size() == 0) {
            // no match possible
           return cherryPicks;
        } else {
            // use matching strategies if no direct match is possible
            cherryPicks = matchCandidates(output1, output2);
        }

        return cherryPicks;
    }

    /**
     * Finds matching commits from two given sets.
     *
     * @param output1   Set of commits eligible for cherry picks from one direction (i.e. git cherry branch1 branch2)
     * @param output2   Set of commits eligible for cherry picks from other direction (git cherry branch2 branch1)
     * @return  Set of CherryPicks with commits for which a match was found
     * @throws GitAPIException
     * @throws IOException
     */

    private List<CherryPick> matchCandidates(Map<String, Commit> output1, Map<String, Commit> output2) throws GitAPIException, IOException {
        List<CherryPick> cherryPicks = new ArrayList<>();

        // Look for cherry picks that can be computed directly from source id given in the commit message
        // Needs to be done in both directions
        cherryPicks.addAll(computeDirectMatches(output1, output2));
        cherryPicks.addAll(computeDirectMatches(output2, output1));
        // Try to match the rest by using the patch id
        cherryPicks.addAll(matchByPatchId(output1, output2));

        return cherryPicks;
    }

    /**
     * Builds a cherry pick from two commits where the target commit
     *         contains the id of the source commit in its commit message
     *         (esp. achieved by -x option for <git cherry-pick>)
     *
     * @param targetCandidates  Commits that could be identified as CherryTarget due to source id in commit message
     * @param sourceCandidates  Commits that could be identified as CherrySource due to being included as a source id in commit message
     * @return  Matches based on cherry pick information in commit messages
     */
    private List<CherryPick> computeDirectMatches(Map<String, Commit> targetCandidates, Map<String, Commit> sourceCandidates){
        List<CherryPick> cherryPicks = new ArrayList<>();
        Collection<Commit> targetCandidatesList = new ArrayList<>(targetCandidates.values());
        for(Commit commit : targetCandidatesList){
            String message = commit.message();

            if(message.contains("cherry picked from commit")){
                // TODO: use more generic way to look for source id e.g. use regex to find SHA1?
                //            (but how could we ensure then that this id refers to a cherry pick?)

                String[] messageComponents = message.split("\s");
                String sourceId = messageComponents[messageComponents.length - 1];
                sourceId = sourceId.strip().replace(")", "");

                if(sourceCandidates.containsKey(sourceId)){
                    Commit source = sourceCandidates.get(sourceId);
                    cherryPicks.add(CherryPick.createCherryPick(source, commit));
                } else {
                    LOGGER.info("Source commit with id " + sourceId +" could not be found!");
                }

                targetCandidates.remove(commit.id());
                sourceCandidates.remove(sourceId);
            }
        }

        // if(cherryPicks.size() != 0) LOGGER.info("Matched " + cherryPicks.size() + " CherryPicks by source id in message");

        return cherryPicks;
    }

    /**
     * Uses messages to build CherryPicks from the remaining commits.
     *
     * @param commits1 Remaining unmatched commits from one branch
     * @param commits2 Remaining unmatched commits from the other branch
     * @return  Successfully matched commits as CherryPicks
     */

    private List<CherryPick> matchByMessageEquality(Map<String, Commit> commits1, Map<String, Commit> commits2) {
        List<CherryPick> cherryPicks = new ArrayList<>();

        // build map from message to commit for one commit list
        Map<String, Commit> message2commit = new HashMap<>();
        for(Commit commit : commits2.values()){
            message2commit.put(commit.message(), commit);
        }

        Collection<Commit> toBeMatched = new ArrayList(commits1.values());
        for(Commit commit : toBeMatched){
            if(message2commit.containsKey(commit.message())){
                Commit matchingCommit = message2commit.get(commit.message());
                cherryPicks.add(CherryPick.determineSourceAndTarget(matchingCommit, commit));
                commits1.remove(commit.id());
                commits2.remove(matchingCommit.id());
            } else {
                LOGGER.info("Matching commit for commit with id " + commit.id() +" could not be found!");
            }
        }

        return cherryPicks;
    }

    /**
     * Uses the patch id (representing the changes in a commit)
     * to find equivalent commits
     *
     * @param commits1 Remaining unmatched commits from one branch
     * @param commits2 Remaining unmatched commits from the other branch
     * @return  Successfully matched commits as CherryPicks
     * @throws GitAPIException
     * @throws IOException
     */

    private List<CherryPick> matchByPatchId(Map<String, Commit> commits1, Map<String, Commit> commits2) throws GitAPIException, IOException {
        Map<String, Commit> patch2commit = new HashMap<>();
        List<CherryPick> cherryPicks = new ArrayList<>();

        for(Commit commit : commits2.values()){
            Optional<String> patchId = repository.getPatchId(commit);
            if(patchId.isPresent()){
                patch2commit.put(patchId.get(), commit);
            }
        }

        Collection<Commit> toBeMatched = new ArrayList(commits1.values());
        for(Commit commit : toBeMatched){
            Optional<String> patchId = repository.getPatchId(commit);
            if(patchId.isPresent()){
                if(patch2commit.containsKey(patchId)){
                    Commit matchingCommit = patch2commit.get(patchId);
                    cherryPicks.add(CherryPick.determineSourceAndTarget(matchingCommit, commit));
                    commits1.remove(commit.id());
                    commits2.remove(matchingCommit.id());
                }
            }

        }

        return cherryPicks;
    }

    /**
     * Uses commit ids to get more information on commits from git repository
     *
     * @param commitIds
     * @return
     * @throws IOException
     */
    private Map<String, Commit> getCommits(List<String> commitIds, Branch branch) throws IOException {
        Map<String, Commit> commits = new HashMap();

        for(String id : commitIds){
            commits.put(id, repository.getCommitHandleById(id, branch));
        }

        return commits;
    }

    /**
     * Enables the use of the command line.
     *
     * @param command
     * @return Output of command, line by line
     * @throws IOException
     */
    private List<String> executeCommand(final String[] command) throws IOException {
        ProcessBuilder pb = new ProcessBuilder(command);
        List<String> output = new ArrayList<>();
        pb.redirectErrorStream(true);
        pb.directory(repository.path().toFile());
        Process p = pb.start();
        BufferedReader reader = new BufferedReader(new InputStreamReader(p.getInputStream()));

        String line;

        while ((line = reader.readLine()) != null) {
            output.add(line);
        }

        reader.close();

        return output;
    }

    /**
        Output of <git cherry> is of the form, e.g.:
        + commit id
        - commit id
        - commit id
        + commit id

        In our case, we are only interested in the ones with '-'
        since they "have an equivalent in <upstream>"
        (see https://git-scm.com/docs/git-cherry)
     */
    private List<String> parseOutput(List<String> output){
        List<String> cherryCandidates = new ArrayList<String>();

        for(String line: output){
            String[] outputComponents = line.split(" ");
            if("-".equals(outputComponents[0])) cherryCandidates.add(outputComponents[1]);
        }

        return cherryCandidates;
    }
}
