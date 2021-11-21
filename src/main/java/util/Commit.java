package util;

import java.util.Date;

public record Commit(String id, Branch branch, String message, Date timestamp) {
    public boolean after(Commit commit){
        return this.timestamp().after(commit.timestamp());
    }

    public boolean before(Commit commit){
        return this.timestamp().before(commit.timestamp());
    }
}
