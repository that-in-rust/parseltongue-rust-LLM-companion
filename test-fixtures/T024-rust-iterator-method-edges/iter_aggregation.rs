fn analyze(data: Vec<i32>) -> (i32, i32, usize) {
    let max = data.iter().max();
    let min = data.iter().min();
    let count = data.iter().count();
    (max.unwrap(), min.unwrap(), count)
}
