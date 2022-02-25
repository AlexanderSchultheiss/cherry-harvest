package util;

import org.eclipse.jgit.api.errors.GitAPIException;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Path;
import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.*;
import java.util.stream.Collectors;

public class RepositoryTest {
    static Repository repo;

    @BeforeAll
    public static void setup() throws IOException, GitAPIException {
        String p = "..\\cherry-test\\repo-test";
        Path path = Path.of(p);
        repo = new Repository(path);
    }

    @Test
    public void testGetCommitByHandleId() throws ParseException, IOException {
        String id = "5ec2718af8376676365b30fd8a358de9a7869280";
        Commit result = repo.getCommitHandleById(id, null);

        SimpleDateFormat ft = new SimpleDateFormat("MM dd HH:mm:ss yyyy");
        Date date = ft.parse("12 22 20:10:51 2021");

        Commit expected = new Commit(id, null, "added main feature\n", date);

        assert expected.equals(result);
    }

    @Test
    public void testPatchIdCherryPicked() throws GitAPIException, IOException, ParseException {
        String id1 = "f3dffce1f2c8cd44068e45cd3220b03012f9e2cf";
        Commit commit1 = repo.getCommitHandleById(id1, null);
        Optional<String> opt1 = repo.getPatchId(commit1);

        String id2 = "ccd0dd3c755c0383f44eb15ca6cec95b335caf4d";
        Commit commit2 = repo.getCommitHandleById(id2, null);
        Optional<String> opt2 = repo.getPatchId(commit2);

        assert opt1.isPresent() && opt2.isPresent() && opt1.get().equals(opt2.get());
    }

    @Test
    public void testPatchIdUnequal() throws GitAPIException, IOException, ParseException {
        String id1 = "f3dffce1f2c8cd44068e45cd3220b03012f9e2cf";
        Commit commit1 = repo.getCommitHandleById(id1, null);
        Optional<String> opt1 = repo.getPatchId(commit1);

        String id2 = "dc3a0543e626cdf98d446cbaf8d97236ecff8f37";
        Commit commit2 = repo.getCommitHandleById(id2, null);
        Optional<String> opt2 = repo.getPatchId(commit2);

        assert opt1.isPresent() && opt2.isPresent() && !opt1.get().equals(opt2.get());
    }

    @Test
    public void testPatchIdEmpty() throws GitAPIException, IOException {
        Commit c = new Commit("6473711111", null, "test", new Date());
        Optional<String> opt = repo.getPatchId(c);

        assert opt.isEmpty();
    }

    @Test
    public void getCommits(){

    }

    @Test
    public void getCommitsWithOneParent(){

    }

    @Test
    public void testCherryPickCandidates() throws GitAPIException, IOException {
        Map<String, Set<Commit>> candidates = repo.computeCherryPickCandidates();
        List<Set<Commit>> list = candidates.values().stream().filter(set -> set.size() >= 2).collect(Collectors.toList());

        assert list.size() == 1;

        Set<Commit> commits = list.get(0);
        Commit[] array = new Commit[2];
        commits.toArray(array);

        assert (array[0].id().equals("f3dffce1f2c8cd44068e45cd3220b03012f9e2cf") && array[1].id().equals("ccd0dd3c755c0383f44eb15ca6cec95b335caf4d")) ||
                (array[1].id().equals("f3dffce1f2c8cd44068e45cd3220b03012f9e2cf") && array[0].id().equals("ccd0dd3c755c0383f44eb15ca6cec95b335caf4d")) ;
    }

    @Test
    public void testGetLocalBranches() throws GitAPIException {
        List<Branch> branches = repo.getBranches(Repository.ListMode.LOCAL);
        assert branches.size() == 3;
    }

    @Test
    public void testGetAllBranches() throws GitAPIException {
        List<Branch> branches = repo.getBranches(Repository.ListMode.ALL);
        assert branches.size() == 3;
    }
}
