function check(items) {
    const found = items.find(x => x.id === 1);
    const hasSome = items.some(x => x.active);
    const allValid = items.every(x => x.valid);
}
