public class Finder {
    public User Find(List<User> users) {
        var first = users.FirstOrDefault();
        var single = users.SingleOrDefault();
        var last = users.Last();
        return first;
    }
}
