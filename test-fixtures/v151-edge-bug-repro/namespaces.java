// v151 Bug Reproduction: Java Qualified Names
// Java uses . for package paths (not :: so LESS affected)
// Included for completeness - Java should work correctly

package com.myapp.services;

import java.util.ArrayList;
import java.util.List;
import java.util.HashMap;
import java.util.Map;

public class UserService {

    public User createUser() {
        // Edge 1: Fully qualified class instantiation
        java.util.ArrayList<String> list = new java.util.ArrayList<>();

        // Edge 2: Qualified static method
        String value = java.lang.System.getenv("PATH");

        // Edge 3: Nested package type
        java.util.Map<String, Object> map = new java.util.HashMap<>();

        // Edge 4: Qualified exception
        try {
            processData();
        } catch (java.lang.RuntimeException e) {
            java.lang.System.err.println(e.getMessage());
        }

        User user = new User();
        processUser(user);
        return user;
    }

    private void processUser(User user) {
        // Edge 5: Static method on qualified class
        java.lang.System.out.println("Processing user");

        // Edge 6: Qualified utility class
        java.util.Objects.requireNonNull(user);

        // Edge 7: Thread from java.lang
        java.lang.Thread.currentThread().getName();
    }

    private void processData() {
        // Simple method
    }
}

class User {
    private String name;

    public String getName() {
        return name;
    }
}
