class DataService {
    async fetchAndProcess() {
        // Constructor calls
        const list = new Array<string>();
        const model = new DataModel();

        // Property access
        const name = model.name;

        // Method calls
        this.helper();

        // Collection operations
        const filtered = list.filter(x => x.length > 0);

        // Async/await
        const data = await fetchData();

        return filtered;
    }

    helper() { }
}
