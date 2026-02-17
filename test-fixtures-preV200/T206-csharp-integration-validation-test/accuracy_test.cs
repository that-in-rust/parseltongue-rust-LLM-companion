using System;
using System.Collections.Generic;

public class Logger {
    public void DoWork() {
        Helper();
    }
    private void Helper() {
        Console.WriteLine("done");
    }
}

public class DataManager {
    public void Create() {
        var list = new List<string>();
        var model = new DataModel();
        list.Add("item");
    }
}

public class DataModel { }
