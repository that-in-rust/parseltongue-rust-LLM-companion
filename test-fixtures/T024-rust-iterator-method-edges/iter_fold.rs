fn calculate(values: Vec<i32>) -> i32 {
    values.iter().fold(0, |acc, x| acc + x)
}
