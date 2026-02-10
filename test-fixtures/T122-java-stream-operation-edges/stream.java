import java.util.List;
import java.util.stream.Collectors;

public class DataService {
    public List<String> getNames(List<User> users) {
        return users.stream()
            .filter(u -> u.isActive())
            .map(u -> u.getName())
            .collect(Collectors.toList());
    }
}
