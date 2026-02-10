class Transformer {
    transform(items: Item[]) {
        const names = items.map(x => x.name);
        const filtered = items.filter(x => x.active);
    }
}
