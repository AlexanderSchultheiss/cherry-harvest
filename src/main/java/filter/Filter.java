package filter;

import cherry.CherryPick;

import java.util.Set;

public interface Filter {
    /**
     * Filters the given cherry pick candidates for cherry picks
     * by a given heuristic (and discards objects that are considered
     * to originate from other git operations like rebase)
     *
     * @param cherryPicks
     * @return set of CherryPick objects that are (still) considered to be cherry picks
     */
    Set<CherryPick> filter(Set<CherryPick> cherryPicks);
}
