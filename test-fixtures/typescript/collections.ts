// Test fixture for TypeScript collection operations detection
// REQ-TYPESCRIPT-003.0

class DataTransformer {
    transform(items: Item[]) {
        // Array methods
        const names = items.map(x => x.name);
        const active = items.filter(x => x.active);
        const total = items.reduce((sum, x) => sum + x.value, 0);

        items.forEach(x => console.log(x));

        const first = items.find(x => x.id === 1);
        const hasActive = items.some(x => x.active);
        const allValid = items.every(x => x.valid);
    }

    chainedOperations(data: Data[]) {
        // Chained operations
        const result = data
            .filter(x => x.valid)
            .map(x => x.value)
            .reduce((a, b) => a + b);

        return result;
    }
}

interface Item {
    id: number;
    name: string;
    active: boolean;
    value: number;
    valid: boolean;
}

interface Data {
    valid: boolean;
    value: number;
}
