// Test fixture for TypeScript constructor detection
// REQ-TYPESCRIPT-001.0

class UserService {
    create() {
        // Simple constructor
        const user = new User();

        // Generic constructor
        const list = new Array<string>();
        const map = new Map<string, User>();

        // Qualified constructor
        const model = new Models.DataModel();
    }
}

class User {
    name: string;
    age: number;
}

namespace Models {
    export class DataModel {
        id: number;
    }
}
