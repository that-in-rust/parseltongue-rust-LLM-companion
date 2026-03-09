function processItems(items) {
    return items
        .filter(x => x.active)
        .map(x => x.name);
}
