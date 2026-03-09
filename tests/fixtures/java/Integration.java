// Java Integration Test Fixture
// Demonstrates: Real-world service with multiple pattern types

package com.example;

import java.util.List;
import java.util.stream.Collectors;
import javax.persistence.Entity;
import javax.persistence.Id;

@Entity
public class Integration {
    @Id
    private Long id;

    private String name;
}

class UserService {
    private UserRepository repository;
    private EmailService emailService;

    // Constructor with dependencies
    public UserService() {
        this.repository = new UserRepository();
        this.emailService = new EmailService();
    }

    // Method with constructor, streams, and generics
    public List<String> getActiveUserEmails() {
        List<User> users = repository.findAll();

        return users.stream()
            .filter(user -> user.isActive())
            .map(user -> user.getEmail())
            .filter(email -> email != null && !email.isEmpty())
            .collect(Collectors.toList());
    }

    // Method with multiple constructors and method calls
    public User createUser(String name, String email) {
        User user = new User();
        user.name = name;
        user.email = email;

        repository.save(user);
        emailService.sendWelcome(email);

        return user;
    }

    // Method with nested constructor calls
    public void setupNotifications() {
        NotificationConfig config = new NotificationConfig(
            new EmailSettings("smtp.example.com", 587),
            new PushSettings("api.example.com")
        );

        NotificationService service = new NotificationService(config);
        service.initialize();
    }

    // Method with generics and stream operations
    public <T extends Comparable<T>> List<T> sortItems(List<T> items) {
        return items.stream()
            .sorted()
            .collect(Collectors.toList());
    }
}

class UserRepository {
    public List<User> findAll() {
        return new ArrayList<>();
    }

    public void save(User user) {
        // save logic
    }
}

class EmailService {
    public void sendWelcome(String email) {
        // email logic
    }
}

class User {
    String name;
    String email;
    boolean active;

    public String getEmail() { return email; }
    public boolean isActive() { return active; }
}

class NotificationConfig {
    NotificationConfig(EmailSettings email, PushSettings push) {}
}

class EmailSettings {
    EmailSettings(String host, int port) {}
}

class PushSettings {
    PushSettings(String apiUrl) {}
}

class NotificationService {
    NotificationService(NotificationConfig config) {}
    void initialize() {}
}
