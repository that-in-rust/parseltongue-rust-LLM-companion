public class Calculator {
    public int Calculate(List<int> numbers) {
        var count = numbers.Count();
        var sum = numbers.Sum();
        var avg = numbers.Average();
        return sum;
    }
}
