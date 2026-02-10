public class ApiClient {
    public async Task GetUserAsync(int id) {
        var response = await this.HttpClient.GetAsync(url);
        var data = await response.Content.ReadAsync();
        var json = await ParseJsonAsync(data);
    }
}
