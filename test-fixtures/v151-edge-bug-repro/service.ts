// v151 Bug Reproduction: TypeScript Zero Edges
// Expected: 5 edges total

class UserService {
    createUser(): void {
        // Edge 1: Generic constructor call
        const list = new Array<string>();

        // Edge 2: Method call on object
        list.push("user1");

        // Edge 3: Method call within class
        this.processUser("test");
    }

    private processUser(name: string): void {
        // Edge 4: Constructor call
        const user = new User();

        // Edge 5: External function call
        console.log(name);
    }
}

class User {
    name: string = "";
}

export { UserService, User };
