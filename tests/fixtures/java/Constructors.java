// Java Constructor Pattern Test Fixture
// Demonstrates: new ClassName(), new GenericClass<T>()

package com.example;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class Constructors {
    // Simple constructor
    public void createSimple() {
        Person person = new Person();
        DataModel model = new DataModel();
    }

    // Generic constructor
    public void createGeneric() {
        ArrayList<String> list = new ArrayList<>();
        HashMap<String, Integer> map = new HashMap<>();
        List<Person> people = new ArrayList<>();
    }

    // Constructor with arguments
    public void createWithArgs() {
        Person person = new Person("John", 30);
        DataModel model = new DataModel(42, "test");
    }

    // Nested constructor calls
    public void createNested() {
        Container container = new Container(new Person("Alice"));
    }

    // Anonymous inner class (edge case)
    public void createAnonymous() {
        Runnable task = new Runnable() {
            @Override
            public void run() {
                System.out.println("Running");
            }
        };
    }
}

class Person {
    String name;
    int age;

    Person() {}
    Person(String name) { this.name = name; }
    Person(String name, int age) {
        this.name = name;
        this.age = age;
    }
}

class DataModel {
    int id;
    String value;

    DataModel() {}
    DataModel(int id, String value) {
        this.id = id;
        this.value = value;
    }
}

class Container {
    Person person;
    Container(Person person) { this.person = person; }
}
