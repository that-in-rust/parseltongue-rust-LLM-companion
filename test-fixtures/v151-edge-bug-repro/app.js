// v151 Bug Reproduction: JavaScript Zero Edges
// Expected: 5 edges total

class UserService {
    createUser() {
        // Edge 1: Constructor call
        const list = new Array();

        // Edge 2: Method call on object
        list.push("user1");

        // Edge 3: Method call within class
        this.processUser("test");
    }

    processUser(name) {
        // Edge 4: Constructor call
        const user = new User();

        // Edge 5: External function call
        console.log(name);
    }
}

class User {}

module.exports = { UserService, User };
