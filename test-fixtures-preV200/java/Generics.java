// Java Generic Type Pattern Test Fixture
// Demonstrates: List<T>, Map<K, V>, custom generics

package com.example;

import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.ArrayList;
import java.util.HashMap;

public class Generics<T> {
    // Generic field declarations
    private List<String> names;
    private Map<String, Integer> counts;
    private Set<Long> ids;

    // Nested generic types
    private Map<String, List<Integer>> groupedData;
    private List<Map<String, Object>> records;

    // Generic method parameters
    public void processItems(List<T> items) {
        for (T item : items) {
            System.out.println(item);
        }
    }

    // Generic return types
    public List<T> getItems() {
        return new ArrayList<>();
    }

    // Multiple generic parameters
    public <K, V> Map<K, V> createMap(K key, V value) {
        Map<K, V> map = new HashMap<>();
        map.put(key, value);
        return map;
    }

    // Bounded generics
    public <E extends Number> List<E> filterNumbers(List<E> numbers) {
        return numbers;
    }

    // Wildcard generics
    public void printList(List<?> items) {
        for (Object item : items) {
            System.out.println(item);
        }
    }

    // Generic constructor
    public Generics(List<T> initialItems) {
        // initialization
    }
}

// Generic class with multiple type parameters
class Pair<K, V> {
    private K key;
    private V value;

    public Pair(K key, V value) {
        this.key = key;
        this.value = value;
    }

    public K getKey() { return key; }
    public V getValue() { return value; }
}

// Generic interface
interface Repository<T> {
    List<T> findAll();
    T findById(Long id);
    void save(T entity);
}
