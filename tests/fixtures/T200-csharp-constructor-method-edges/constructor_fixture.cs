using System.Collections.Generic;

public class DataManager {
    public void Create() {
        var list = new List<string>();
        var model = new DataModel();
        list.Add("item");
    }
}

public class DataModel {
    public string Name { get; set; }
}
