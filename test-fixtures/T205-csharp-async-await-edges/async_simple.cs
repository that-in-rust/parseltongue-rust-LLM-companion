public class AsyncService {
    public async Task ProcessAsync() {
        var data = await FetchDataAsync();
        await SaveAsync(data);
    }
}
