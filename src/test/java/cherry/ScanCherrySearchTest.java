package cherry;

import org.eclipse.jgit.api.errors.GitAPIException;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import util.Commit;
import util.Repository;

import java.io.IOException;
import java.nio.file.Path;
import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.Calendar;
import java.util.Date;
import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.TimeUnit;

public class ScanCherrySearchTest {
    private static ScanCherrySearch dummyCherrySearch;

    @BeforeAll
    public static void setup() throws IOException {
        dummyCherrySearch = new ScanCherrySearch(null);
    }

    @Test
    public void testMatchOldest() throws InterruptedException {
        Commit old = new Commit("old", null, "oldest", Calendar.getInstance().getTime());
        TimeUnit.SECONDS.sleep(5);
        Commit new1 = new Commit("new1", null, "newer", Calendar.getInstance().getTime());
        TimeUnit.SECONDS.sleep(5);
        Commit new2 = new Commit("new2", null, "newest", Calendar.getInstance().getTime());
        Set<Commit> commits = new HashSet<>();
        commits.add(old);
        commits.add(new2);
        commits.add(new1);

        Set<CherryPick> cherryPicks = dummyCherrySearch.matchOldestWithRest(commits);
        Set<CherryPick> expected = new HashSet<>();
        CherrySource source = new CherrySource(old);

        expected.add(new CherryPick(source, new CherryTarget(new1)));
        expected.add(new CherryPick(source, new CherryTarget(new2)));

        assert expected.equals(cherryPicks);
    }

    @Test
    public void testMatchOldestWithEmpty() throws InterruptedException {
        Set<Commit> commits = new HashSet<>();

        Set<CherryPick> cherryPicks = dummyCherrySearch.matchOldestWithRest(commits);
        Set<CherryPick> expected = new HashSet<>();

        assert expected.equals(cherryPicks);
    }

    @Test
    public void testMatchOldestWithOne() throws InterruptedException {
        Commit old = new Commit("old", null, "oldest", Calendar.getInstance().getTime());
        Set<Commit> commits = new HashSet<>();
        commits.add(old);

        Set<CherryPick> cherryPicks = dummyCherrySearch.matchOldestWithRest(commits);
        Set<CherryPick> expected = new HashSet<>();

        assert expected.equals(cherryPicks);
    }

    @Test
    public void testMatchAllWithOne() throws InterruptedException {
        Commit old = new Commit("old", null, "oldest", Calendar.getInstance().getTime());
        Set<Commit> commits = new HashSet<>();
        commits.add(old);

        Set<CherryPick> cherryPicks = dummyCherrySearch.matchAll(commits);
        Set<CherryPick> expected = new HashSet<>();

        assert expected.equals(cherryPicks);
    }

    @Test
    public void testMatchAllWithEmpty() throws InterruptedException {
        Set<Commit> commits = new HashSet<>();
        Set<CherryPick> cherryPicks = dummyCherrySearch.matchAll(commits);
        Set<CherryPick> expected = new HashSet<>();

        assert expected.equals(cherryPicks);
    }

    @Test
    public void testMatchAll() throws InterruptedException {
        Commit old = new Commit("old", null, "oldest", Calendar.getInstance().getTime());
        TimeUnit.SECONDS.sleep(5);
        Commit new1 = new Commit("new1", null, "newer", Calendar.getInstance().getTime());
        TimeUnit.SECONDS.sleep(5);
        Commit new2 = new Commit("new2", null, "newest", Calendar.getInstance().getTime());
        Set<Commit> commits = new HashSet<>();
        commits.add(old);
        commits.add(new2);
        commits.add(new1);

        Set<CherryPick> cherryPicks = dummyCherrySearch.matchAll(commits);
        Set<CherryPick> expected = new HashSet<>();


        expected.add(new CherryPick(new CherrySource(old), new CherryTarget(new1)));
        expected.add(new CherryPick(new CherrySource(old), new CherryTarget(new2)));
        expected.add(new CherryPick(new CherrySource(new1), new CherryTarget(new2)));

        assert expected.equals(cherryPicks);
    }

    @Test
    public void testCherrySearchOnEmptyRepo() throws IOException, GitAPIException {
        Path path = Path.of("D:/Maike/git/cherry-test/empty-repo");
        Set<CherryPick> cherryPicks;
        Set<CherryPick> expected;

        try(Repository repo = new Repository(path)) {
            CherrySearch cherrySearch = new ScanCherrySearch(repo);
            cherryPicks = cherrySearch.findAllCherryPicks();
            expected = new HashSet<>();
        }

        assert expected.equals(cherryPicks);
    }

    @Test
    public void testCherrySearchOnRepoWithoutCherryPicks() throws IOException, GitAPIException {
        Path path = Path.of("D:/Maike/git/cherry-test/no-cherries");
        Set<CherryPick> cherryPicks;
        Set<CherryPick> expected;

        try(Repository repo = new Repository(path)) {
            CherrySearch cherrySearch = new ScanCherrySearch(repo);
            cherryPicks = cherrySearch.findAllCherryPicks();
            expected = new HashSet<>();
        }

        assert expected.equals(cherryPicks);
    }

    @Test
    public void testCherrySearchOnRepoWithCherryPicks() throws IOException, GitAPIException, ParseException {
        Path path = Path.of("D:/Maike/git/cherry-test/several-cherries");
        Set<CherryPick> cherryPicks;

        try(Repository repo = new Repository(path)) {
            CherrySearch cherrySearch = new ScanCherrySearch(repo);
            cherryPicks = cherrySearch.findAllCherryPicks();
        }

        SimpleDateFormat ft = new SimpleDateFormat("MM dd HH:mm:ss yyyy");

        Set<CherryPick> expected = new HashSet<>();
        Date src_d1 = ft.parse("12 13 17:39:07 2021");
        Commit src1 = new Commit("43f85e7816f72e2e035d1846163ef61867a51585", null, "added file2\n", src_d1);
        Date tgt_d1 = ft.parse("12 13 17:41:05 2021");
        Commit tgt1 = new Commit("7313f8c30cdfce82151e2bc0ea89a1a2a79062f2", null, "added file2\n", tgt_d1);

        expected.add(new CherryPick(new CherrySource(src1), new CherryTarget(tgt1)));


        Date src_d2 = ft.parse("12 13 17:40:11 2021");
        Commit src2 = new Commit("cbf5792876be4b6b766b9edbda380a233a77beef", null, "added ccc to file1\n", src_d2);
        Date tgt_d2 = ft.parse("12 13 17:40:49 2021");
        Commit tgt2 = new Commit("01e0579d9e3d8b0f78e94ceadc1a055878e31120", null, "added ccc to file1\n", tgt_d2);

        expected.add(new CherryPick(new CherrySource(src2), new CherryTarget(tgt2)));

        assert expected.equals(cherryPicks);
    }
}
