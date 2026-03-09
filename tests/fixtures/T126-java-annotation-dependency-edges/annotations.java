import javax.persistence.Entity;
import javax.persistence.Id;

@Entity
@Table(name = "users")
public class User {
    @Id
    @GeneratedValue
    private Long id;

    @Override
    public String toString() {
        return "User";
    }
}
