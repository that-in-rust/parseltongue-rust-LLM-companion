fn search(items: &[Item]) -> bool {
    items.iter().find(|x| x.id == 10);
    items.iter().any(|x| x.active);
    items.iter().all(|x| x.valid);
    true
}
