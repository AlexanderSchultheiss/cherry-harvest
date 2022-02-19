package util;

import java.util.Date;
import java.util.Objects;

/**
 * Representation of git commit object
 * that also enables time-based comparison of commits
 *
 * @author Maike
 */

public record Commit(String id, Branch branch, String message, Date timestamp) {

    /**
     * Determines whether this commit was created after the given commit
     *
     * @param commit    Commit to compare to
     * @return  true if given commit is older than this commit, otherwise false
     */
    public boolean after(Commit commit){
        return this.timestamp().after(commit.timestamp());
    }

    /**
     * Determines whether this commit was created before the given commit
     *
     * @param commit    Commit to compare to
     * @return  true if this commit is older than given commit, otherwise false
     */
    public boolean before(Commit commit){
        return this.timestamp().before(commit.timestamp());
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        Commit commit = (Commit) o;
        // only change to default implementation: branch is excluded
        // id only would be sufficient?
        return Objects.equals(id, commit.id) && Objects.equals(timestamp, commit.timestamp()) && Objects.equals(message, commit.message);
    }

    @Override
    public int hashCode() {
        // only change to default implementation: branch is excluded
        // id only would be sufficient?
        return Objects.hash(id, message, timestamp);
    }
}
