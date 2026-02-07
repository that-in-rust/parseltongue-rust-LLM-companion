// v151 Bug Reproduction: C# Zero Edges
// Expected: 5 edges total

using System.Collections.Generic;

public class UserService
{
    public void CreateUser()
    {
        // Edge 2: Constructor call
        var list = new List<string>();

        // Edge 3: Method call on object
        list.Add("user1");

        // Edge 4: Another method call
        ProcessUser("test");
    }

    private void ProcessUser(string name)
    {
        // Edge 5: Constructor call
        var user = new User();
    }
}

public class User { }
