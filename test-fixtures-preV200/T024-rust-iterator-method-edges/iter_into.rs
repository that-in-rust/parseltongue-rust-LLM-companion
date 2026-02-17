fn process(vec: Vec<u32>) -> u32 {
    vec.into_iter()
        .filter(|&x| x > 0)
        .sum()
}
