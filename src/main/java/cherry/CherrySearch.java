package cherry;

import cherry.CherrySource;
import cherry.CherryTarget;
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

    public List<CherryPick> findAllCherryPicks(){

        // go through all pairs of branches? or can we somehow use git semantics to limit pairs of branches?
        return new ArrayList<>();
    }

    public List<CherryPick> findCherryPicks(Branch upstream, Branch head) throws IOException, GitAPIException {
        // https://stackoverflow.com/questions/2922652/git-is-there-a-way-to-figure-out-where-a-commit-was-cherry-picked-from
        // call git cherry upstream head -> record '-' output
        // call git cherry head upstream -> record '-' output
        // need to match those -> either try to find commit messages including id of original commit (when -x option used)
        // or use commit messages to match candidates?
        // when matched, determine which is source and which target by checking time stamps?
        List<CherryPick> cherries = new ArrayList<>();

        // call git cherry upstream head -> record '-' output
        final List<String> upstr2head =  cherry(upstream, head);
        // call git cherry head upstream -> record '-' output
        final List<String> head2upstr = cherry(head, upstream);

        Map<String, Commit> commitsHead2Upstr = getCommits(head2upstr);
        Map<String, Commit> commitsUpstr2Head = getCommits(upstr2head);

        cherries = computeCherryPicks(commitsHead2Upstr, commitsUpstr2Head);

        return cherries;
    }

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
        } else {
            cherryPicks = matchCandidates(output1, output2);
        }
        return cherryPicks;
    }

    private List<CherryPick> matchCandidates(Map<String, Commit> output1, Map<String, Commit> output2) throws GitAPIException, IOException {
        List<CherryPick> cherryPicks = new ArrayList<>();
        cherryPicks.addAll(computeDirectMatches(output1, output2));
        cherryPicks.addAll(computeDirectMatches(output2, output1));
        cherryPicks.addAll(matchByPatchId(output1, output2));

        return cherryPicks;
    }

    /**

     */
    private List<CherryPick> computeDirectMatches(Map<String, Commit> targetCandidates, Map<String, Commit> sourceCandidates){
        List<CherryPick> cherryPicks = new ArrayList<>();
        Collection<Commit> targetCandidatesList = new ArrayList<>(targetCandidates.values());
        for(Commit commit : targetCandidatesList){
            String message = commit.message();

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


    private CherryPick determineSourceAndTarget(Commit commit1, Commit commit2){
        return commit1.after(commit2)? createCherryPick(commit2, commit1) : createCherryPick(commit1, commit2);
    }

    private CherryPick createCherryPick(Commit src, Commit target){
        return new CherryPick(new CherrySource(src), new CherryTarget(target));
    }

    private Map<String, Commit> getCommits(List<String> commitIds) throws IOException {
        Map<String, Commit> commits = new HashMap();

        for(String id : commitIds){
            commits.put(id, repository.getCommitById(id));
        }

        return commits;
    }

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

    private List<String> parseOutput(List<String> output){
        List<String> cherryCandidates = new ArrayList<String>();

        for(String line: output){
            String[] outputComponents = line.split(" ");
            if("-".equals(outputComponents[0])) cherryCandidates.add(outputComponents[1]);
        }

        return cherryCandidates;
    }
}
