class Pipeline {
    process(data: Data[]) {
        const result = data
            .filter(x => x.valid)
            .map(x => x.value)
            .reduce((a, b) => a + b);
    }
}
