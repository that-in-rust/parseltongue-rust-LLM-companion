public class TaskManager {
    public async Task RunAsync() {
        await Task.Delay(1000);
        await Task.WhenAll(task1, task2);
        await Task.WhenAny(task3, task4);
    }
}
