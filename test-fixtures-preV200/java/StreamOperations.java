// Java Stream API Pattern Test Fixture
// Demonstrates: .stream(), .filter(), .map(), .collect()

package com.example;

import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

public class StreamOperations {
    // Basic stream operations
    public List<String> getNames(List<User> users) {
        return users.stream()
            .map(User::getName)
            .collect(Collectors.toList());
    }

    // Chained stream operations
    public List<String> getActiveUserNames(List<User> users) {
        return users.stream()
            .filter(u -> u.isActive())
            .map(u -> u.getName())
            .collect(Collectors.toList());
    }

    // Complex stream pipeline
    public Set<String> getUniqueEmails(List<User> users) {
        return users.stream()
            .filter(User::isActive)
            .map(User::getEmail)
            .filter(email -> email != null)
            .collect(Collectors.toSet());
    }

    // Stream with forEach
    public void printAllUsers(List<User> users) {
        users.stream()
            .forEach(user -> System.out.println(user.getName()));
    }

    // Stream with reduce
    public int getTotalAge(List<User> users) {
        return users.stream()
            .mapToInt(User::getAge)
            .sum();
    }

    // Stream with findFirst
    public User findFirstActive(List<User> users) {
        return users.stream()
            .filter(User::isActive)
            .findFirst()
            .orElse(null);
    }

    // Stream with anyMatch
    public boolean hasActiveUsers(List<User> users) {
        return users.stream()
            .anyMatch(User::isActive);
    }

    // Parallel stream
    public List<String> processParallel(List<User> users) {
        return users.parallelStream()
            .map(User::getName)
            .collect(Collectors.toList());
    }
}

class User {
    private String name;
    private String email;
    private int age;
    private boolean active;

    public String getName() { return name; }
    public String getEmail() { return email; }
    public int getAge() { return age; }
    public boolean isActive() { return active; }
}
