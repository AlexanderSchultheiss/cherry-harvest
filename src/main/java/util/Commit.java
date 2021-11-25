package util;

import java.util.Date;
import java.util.Objects;

public record Commit(String id, Branch branch, String message, Date timestamp) {

    public boolean after(Commit commit){
        return this.timestamp().after(commit.timestamp());
    }

    public boolean before(Commit commit){
        return this.timestamp().before(commit.timestamp());
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        Commit commit = (Commit) o;
        return Objects.equals(id, commit.id);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, message, timestamp);
    }
}
