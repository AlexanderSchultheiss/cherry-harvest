import util.Branch;
import util.Commit;
import util.Repository;
import util.UnequalCherryCandidatesException;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

public class CherrySearch {
    private Repository repository;

    public CherrySearch(Path path) throws IOException {
        repository = new Repository(path);
    }

    public List<CherryPick> findAllCherryPicks(final Path pathToRepo){
        // go through all pairs of branches? or can we somehow use git semantics to limit pairs of branches?
        return new ArrayList<>();
    }

    public List<CherryPick> findCherryPicks(Branch upstream, Branch head) throws IOException, UnequalCherryCandidatesException{
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

        if(head2upstr.size() != upstr2head.size()){
            throw new UnequalCherryCandidatesException();
        }

        List<Commit> commitsHead2Upstr = getCommits(head2upstr);
        List<Commit> commitsUpstr2Head = getCommits(upstr2head);
        cherries = matchCandidates(commitsHead2Upstr, commitsUpstr2Head);

        return cherries;
    }

    private List<String> cherry(Branch upstream, Branch head) throws IOException {
        final String[] command = {"util", "cherry", upstream.name(), head.name()};
        return parseOutput(executeCommand(command));
    }

    private List<CherryPick> matchCandidates(List<Commit> output1, List<Commit> output2) {
        List<CherryPick> cherryPicks = new ArrayList<>();

        if(output1.size() == 1 && output2.size() == 1){
            // immediate match, just need to figure out source and target
            cherryPicks.add(determineSourceAndTarget(output1.get(0), output2.get(0));
        }
        return cherryPicks;
    }

    private CherryPick determineSourceAndTarget(Commit commit1, Commit commit2){
        return commit1.after(commit2)? createCherryPick(commit2, commit1) : createCherryPick(commit1, commit2);
    }

    private CherryPick createCherryPick(Commit src, Commit target){
        return new CherryPick(new CherrySource(src), new CherryTarget(target));
    }

    private List<Commit> getCommits(List<String> commitIds) throws IOException {
        List<Commit> commits = new ArrayList<>();

        for(String id : commitIds){
            commits.add(repository.getCommitById(id));
        }

        return commits;
    }

    private static List<String> executeCommand(final String[] command) throws IOException {
        ProcessBuilder pb = new ProcessBuilder(command);
        List<String> output = new ArrayList<>();
        pb.redirectErrorStream(true);
        Process p = pb.start();
        BufferedReader reader = new BufferedReader(new InputStreamReader(p.getInputStream()));

        String line;

        while ((line = reader.readLine()) != null) {
            output.add(line);
        }

        reader.close();

        return output;
    }

    private static List<String> parseOutput(List<String> output){
        List<String> cherryCandidates = new ArrayList<String>();

        for(String line: output){
            String[] outputComponents = line.split(" ");
            if("-".equals(outputComponents[0])) cherryCandidates.add(outputComponents[1]);
        }

        return cherryCandidates;
    }
}
