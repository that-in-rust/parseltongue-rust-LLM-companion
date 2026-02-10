public class SetProcessor {
    public List<int> Process(List<int> a, List<int> b) {
        var distinct = a.Distinct();
        var union = a.Union(b);
        var intersect = a.Intersect(b);
        return distinct.ToList();
    }
}
