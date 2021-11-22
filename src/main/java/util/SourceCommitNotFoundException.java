package util;

public class SourceCommitNotFoundException extends Exception{
    @Override
    public String getMessage() {
        return "Could not find source commit - perhaps it does not exist anymore.";
    }
}
