package filter;

import cherry.CherryPick;
import filter.Filter;

import java.util.*;

/**
 * TimeFilter filters CherryPick objects based on time difference between commits
 * as a heuristic to distinguish rebase and cherry pick
 *
 * @author Maike
 */

// TODO: Implement filter based on parent relation instead,
//          as this might not work as expected, since we only have timestamps of original commit time!

public class TimeFilter implements Filter {
    long diffThreshold; // Default difference in milliseconds between two commits to be considered for rebase

    public TimeFilter(long diff){
        this.diffThreshold = diff;
    }

    public TimeFilter(){
        this(60000);
    }

    public Set<CherryPick> filter(Set<CherryPick> cherryPicks){
        Set<CherryPick> filtered = new HashSet<CherryPick>();

        List<CherryPick> sorted = cherryPicks.stream().sorted(new CommitTimeComparator()).toList();
        CherryPick[] cpArray = new CherryPick[sorted.size()];
        sorted.toArray(cpArray);

        for(int i = 0; i < cpArray.length; ++i){
            CherryPick current = cpArray[i];
            boolean rebase = false;

            // check in both directions whether commit builds a chain with successor or predecessor (time-wise)
            if(i-1 >= 0){
                CherryPick previous = cpArray[i-1];
                rebase = timeDifferenceBelowThreshold(previous, current)? true : false;
            }

            if(i+1 < cpArray.length){
                CherryPick next = cpArray[i+1];
                rebase = timeDifferenceBelowThreshold(current, next)? true : false;
            }

            // if not the case -> this could be a true cherry pick
            if(!rebase){
                filtered.add(current);
            }
        }

        return filtered;
    }

    private class CommitTimeComparator implements Comparator<CherryPick> {
        @Override
        public int compare(CherryPick cp1, CherryPick cp2) {
            return cp1.target().commit().timestamp().compareTo(cp2.target().commit().timestamp());
        }
    }

    private boolean timeDifferenceBelowThreshold(CherryPick first, CherryPick second){
        long timeFirst = first.target().commit().timestamp().getTime();
        long timeSecond = second.target().commit().timestamp().getTime();

        return timeSecond - timeFirst < diffThreshold;
    }
}
