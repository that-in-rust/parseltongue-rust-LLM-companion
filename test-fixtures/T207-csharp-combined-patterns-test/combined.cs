public class CompleteService {
    public async Task<List<string>> ProcessAsync(List<User> users) {
        // Property access
        var count = users.Count;

        // LINQ operations
        var active = users
            .Where(u => u.Active)
            .OrderBy(u => u.Name)
            .ToList();

        // Async/await
        var data = await FetchDataAsync();

        // Constructor
        var result = new List<string>();

        // Property assignment
        result.Capacity = count;

        return result;
    }
}
