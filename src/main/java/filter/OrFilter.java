package filter;

import cherry.CherryPick;
import filter.Filter;

import java.util.Set;

/**
 * OrFilter allows to combine the result sets of two filters
 *
 * @author Maike
 */
public class OrFilter implements Filter {
    Filter left;
    Filter right;

    public OrFilter(Filter filter1, Filter filter2){
        this.left = filter1;
        this.right = filter2;
    }

    public Set<CherryPick> filter(Set<CherryPick> cherryPicks){
        Set<CherryPick> filtered = left.filter(cherryPicks);
        filtered.addAll(right.filter(cherryPicks));

        return filtered;
    }
}
