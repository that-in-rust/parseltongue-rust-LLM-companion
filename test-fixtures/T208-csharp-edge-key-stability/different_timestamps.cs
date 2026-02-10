public class Logger {
    public void DoWork() {
        Helper();
    }

    public void Process() {
        DoWork();
    }

    private void Helper() { }
}
