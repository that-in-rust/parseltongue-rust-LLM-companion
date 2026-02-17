public class DataService {
    public List<string> GetNames(List<User> users) {
        return users
            .Where(u => u.Active)
            .Select(u => u.Name)
            .ToList();
    }
}
