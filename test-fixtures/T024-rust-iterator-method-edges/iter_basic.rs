fn transform(items: Vec<Item>) -> Vec<String> {
    items.iter()
        .filter(|x| x.active)
        .map(|x| x.name.clone())
        .collect()
}
