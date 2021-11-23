package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;
import util.*;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Path;
import java.util.*;

public class CherrySearch {
    private Repository repository;

    public CherrySearch(Path path) throws IOException {
        repository = new Repository(path);
    }

    /**
        Finds all cherry picks in the given repository
        by computing cherry picks for all relevant pairs of branches
     */

    public List<CherryPick> findAllCherryPicks() throws GitAPIException, IOException {
        final ArrayList<Branch> branches = new ArrayList<>(repository.getLocalBranches());
        List<CherryPick> cherryPicks = new ArrayList<>();

        for(int i = 0; i < branches.size()-1 ; ++i){
            Branch upstream =  branches.get(i);

            for(Branch head : branches.subList(i+1, branches.size())){
                cherryPicks.addAll(findCherryPicks(upstream, head));
            }
        }

        return cherryPicks;
    }

    /**
     * Finds cherry picks in two given (distinct) branches
     *
     * @param upstream  Branch that may hold commits representing cherry sources or targets
     * @param head      Branch that may hold commits representing cherry sources or targets
     * @return          Cherry picks that were identified
     * @throws IOException
     * @throws GitAPIException
     */

    // see https://stackoverflow.com/questions/2922652/git-is-there-a-way-to-figure-out-where-a-commit-was-cherry-picked-from :
    // call git cherry upstream head -> record '-' output
    // call git cherry head upstream -> record '-' output
    // need to match those -> either try to find commit messages including id of original commit (when -x option used)
    // or use patch id / commit messages to match candidates
    // when matched, determine which is source and which target by checking time stamps

    public List<CherryPick> findCherryPicks(Branch upstream, Branch head) throws IOException, GitAPIException {
        List<CherryPick> cherries = new ArrayList<>();

        final List<String> upstr2head =  cherry(upstream, head);
        final List<String> head2upstr = cherry(head, upstream);

        Map<String, Commit> commitsHead2Upstr = getCommits(head2upstr);
        Map<String, Commit> commitsUpstr2Head = getCommits(upstr2head);

        cherries = computeCherryPicks(commitsHead2Upstr, commitsUpstr2Head);

        return cherries;
    }

    /*
        "Wrapper" for shell command <git cherry upstream head>

        @return Relevant output that has been parsed to be used within this program
     */
    private List<String> cherry(Branch upstream, Branch head) throws IOException {
        final String[] command = {"git", "cherry", upstream.name(), head.name()};
        return parseOutput(executeCommand(command));
    }

    private List<CherryPick> computeCherryPicks(Map<String,Commit> output1, Map<String, Commit> output2) throws GitAPIException, IOException {
        List<CherryPick> cherryPicks = new ArrayList<>();

        if(output1.size() == 1 && output2.size() == 1){
            // immediate match, just need to figure out source and target
            Commit commit1 = output1.values().iterator().next();
            Commit commit2 = output2.values().iterator().next() ;
            cherryPicks.add(determineSourceAndTarget(commit1, commit2));
        } else if(output1.size() == 0 || output2.size() == 0) {
           return cherryPicks;
        } else {
            cherryPicks = matchCandidates(output1, output2);
        }

        return cherryPicks;
    }

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
     * @param sourceCandidates  Commits that could be identified as CherrySource due to being included as a source in commit message
     * @return  Matches based on cherry pick information in commit messages
     */
    private List<CherryPick> computeDirectMatches(Map<String, Commit> targetCandidates, Map<String, Commit> sourceCandidates){
        List<CherryPick> cherryPicks = new ArrayList<>();
        Collection<Commit> targetCandidatesList = new ArrayList<>(targetCandidates.values());
        for(Commit commit : targetCandidatesList){
            String message = commit.message();

            // TODO: use more generic way to look for source id e.g. use regex to find 40 chars long SHA1?
            //            (but how could we ensure then that this id refers to a cherry pick?)

            if(message.contains("cherry picked from commit")){
                String[] messageComponents = message.split("\s");
                String sourceId = messageComponents[messageComponents.length - 1];
                sourceId = sourceId.strip().replace(")", "");

                if(sourceCandidates.containsKey(sourceId)){
                    Commit source = sourceCandidates.get(sourceId);
                    cherryPicks.add(createCherryPick(source, commit));
                } else {
                    System.out.println("Source commit with id " + sourceId +" could not be found!");
                }

                targetCandidates.remove(commit.id());
                sourceCandidates.remove(sourceId);
            }
        }

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
                cherryPicks.add(determineSourceAndTarget(matchingCommit, commit));
                commits1.remove(commit.id());
                commits2.remove(matchingCommit.id());
            } else {
                System.out.println("Matching commit for commit with id " + commit.id() +" could not be found!");
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
            String patchId = repository.getPatchId(commit);
            patch2commit.put(patchId, commit);
        }

        Collection<Commit> toBeMatched = new ArrayList(commits1.values());
        for(Commit commit : toBeMatched){
            String patchId = repository.getPatchId(commit);

            if(patch2commit.containsKey(patchId)){
                Commit matchingCommit = patch2commit.get(patchId);
                cherryPicks.add(determineSourceAndTarget(matchingCommit, commit));
                commits1.remove(commit.id());
                commits2.remove(matchingCommit.id());
            }
        }

        return cherryPicks;
    }

    /**
     * Determines CherrySource and CherryTarget based on timestamps
     */
    private CherryPick determineSourceAndTarget(Commit commit1, Commit commit2){
        return commit1.after(commit2)? createCherryPick(commit2, commit1) : createCherryPick(commit1, commit2);
    }

    /**
        Computes new CherryPick from given source commit and target commit
     */
    private CherryPick createCherryPick(Commit src, Commit target){
        return new CherryPick(new CherrySource(src), new CherryTarget(target));
    }

    /**
     * Uses commit ids to get more information on commits from git repository
     *
     * @param commitIds
     * @return
     * @throws IOException
     */
    private Map<String, Commit> getCommits(List<String> commitIds) throws IOException {
        Map<String, Commit> commits = new HashMap();

        for(String id : commitIds){
            commits.put(id, repository.getCommitById(id));
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
