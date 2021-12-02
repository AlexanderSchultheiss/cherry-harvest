package cherry;

import cherry.CherrySource;
import cherry.CherryTarget;
import util.Commit;

public record CherryPick(CherrySource source, CherryTarget target) {
    /**
     * Determines CherrySource and CherryTarget based on timestamps
     */
    public static CherryPick determineSourceAndTarget(Commit commit1, Commit commit2){
        return commit1.after(commit2)? createCherryPick(commit2, commit1) : createCherryPick(commit1, commit2);
    }

    /**
     Computes new CherryPick from given source commit and target commit
     */
    public static CherryPick createCherryPick(Commit src, Commit target){
        return new CherryPick(new CherrySource(src), new CherryTarget(target));
    }
}
