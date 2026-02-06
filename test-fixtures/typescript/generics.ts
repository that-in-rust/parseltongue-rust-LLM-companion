// Test fixture for TypeScript generic type detection
// REQ-TYPESCRIPT-005.0

class Container<T> {
    // Generic type annotations
    items: Array<T>;
    map: Map<string, T>;
    set: Set<T>;
}

class Repository<T, K> {
    private cache: Map<K, T>;

    constructor() {
        this.cache = new Map<K, T>();
    }

    add(key: K, value: T): void {
        this.cache.set(key, value);
    }

    get(key: K): T | undefined {
        return this.cache.get(key);
    }
}

// Generic function
function process<T>(items: Array<T>): Set<T> {
    return new Set(items);
}

// Generic type alias
type AsyncResult<T> = Promise<T>;
type Dictionary<T> = Map<string, T>;
