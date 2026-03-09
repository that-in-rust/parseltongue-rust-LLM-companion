import java.util.List;
import java.util.stream.Collectors;

public class UserService {
    private UserRepository repository;

    public UserService() {
        this.repository = new UserRepository();
    }

    public List<String> getActiveUserNames() {
        return repository.findAll().stream()
            .filter(user -> user.isActive())
            .map(user -> user.getName())
            .collect(Collectors.toList());
    }

    public User createUser(String name) {
        User user = new User();
        user.name = name;
        return repository.save(user);
    }
}
