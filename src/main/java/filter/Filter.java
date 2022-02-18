package filter;

import cherry.CherryPick;

import java.util.Set;

public interface Filter {
    Set<CherryPick> filter(Set<CherryPick> cherryPicks);
}
