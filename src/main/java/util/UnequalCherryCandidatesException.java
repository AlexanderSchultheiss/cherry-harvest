package util;

public class UnequalCherryCandidatesException extends Exception {
    @Override
    public String getMessage() {
        return "Size of candidates sets is asymmetric.";
    }
}
