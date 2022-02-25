package util;

import org.junit.jupiter.api.Test;

import java.util.Date;
import java.util.concurrent.TimeUnit;

public class CommitTest {
    @Test
    public void testAfterTrue() throws InterruptedException {
        Commit c1 = new Commit("id1", null, "id1", new Date());
        TimeUnit.SECONDS.sleep(5);
        Commit c2 = new Commit("id1", null, "id1", new Date());

        assert c2.after(c1);
        assert !c2.before(c1);
    }

    @Test
    public void testBeforeTrue() throws InterruptedException {
        Commit c1 = new Commit("id1", null, "id1", new Date());
        TimeUnit.SECONDS.sleep(5);
        Commit c2 = new Commit("id1", null, "id1", new Date());

        assert !c1.after(c2);
        assert c1.before(c2);
    }

    @Test
    public void testEqualsTrue() throws InterruptedException {
        Date d = new Date();
        Commit c1 = new Commit("id1", null, "id1", d);
        TimeUnit.SECONDS.sleep(5);
        Commit c2 = new Commit("id1", null, "id1", d);

        assert c1.equals(c2);
        assert c1.equals(c1);
    }

    @Test
    public void testEqualsFalse() throws InterruptedException {
        Date d = new Date();
        Commit c1 = new Commit("id1", null, "message", d);
        TimeUnit.SECONDS.sleep(5);
        Commit c2 = new Commit("id2", null, "message", d);

        assert !c1.equals(c2);
    }
}
