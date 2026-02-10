public class Sorter {
    public List<User> Sort(List<User> users) {
        return users
            .OrderBy(u => u.Name)
            .ThenBy(u => u.Age)
            .ToList();
    }
}
