// Java Annotation Pattern Test Fixture
// Demonstrates: @Override, @Entity, @Autowired, custom annotations

package com.example;

import javax.persistence.Entity;
import javax.persistence.Table;
import javax.persistence.Id;
import javax.persistence.GeneratedValue;
import javax.persistence.Column;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.stereotype.Repository;

// Class-level annotations
@Entity
@Table(name = "users")
public class Annotations {
    // Field-level annotations
    @Id
    @GeneratedValue
    @Column(name = "id")
    private Long id;

    @Column(name = "name", nullable = false)
    private String name;

    // Method-level annotations
    @Override
    public String toString() {
        return "User: " + name;
    }

    // Custom annotation with parameters
    @Deprecated(since = "1.0", forRemoval = true)
    public void oldMethod() {
        // deprecated logic
    }
}

// Service with Spring annotations
@Service
class UserService {
    @Autowired
    private UserRepository repository;

    @Autowired
    public UserService(UserRepository repository) {
        this.repository = repository;
    }

    public void processUser() {
        // service logic
    }
}

// Repository interface
@Repository
interface UserRepository {
    User findById(Long id);
    void save(User user);
}

// Custom annotations
@interface CustomAnnotation {
    String value();
    int priority() default 0;
}

@CustomAnnotation(value = "test", priority = 1)
class AnnotatedClass {
    // annotated class
}
