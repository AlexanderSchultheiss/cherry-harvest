package filter;

import cherry.CherryPick;
import cherry.CherryTarget;
import filter.Filter;

import java.util.HashSet;
import java.util.Set;

public class MessageFilter implements Filter {
    public Set<CherryPick> filter(Set<CherryPick> cherryPicks){
        Set<CherryPick> filtered = new HashSet<CherryPick>();
        for(CherryPick cp : cherryPicks){
            CherryTarget target = cp.target();
            String message = target.commit().message();

            if(message.contains("cherry picked from commit")){
                filtered.add(cp);
            }
        }
        return filtered;
    }
}
