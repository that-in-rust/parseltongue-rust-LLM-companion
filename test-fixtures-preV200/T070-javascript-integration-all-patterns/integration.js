class DataService {
    async fetchAndProcess(id) {
        // Constructor call
        const logger = new Logger();

        // Async/await
        const response = await fetch('/api/data/' + id);
        const data = await response.json();

        // Property access
        const items = data.items;

        // Array methods
        const processed = items
            .filter(x => x.active)
            .map(x => x.value);

        // Promise chain
        return this.save(processed)
            .then(result => result.id)
            .catch(err => console.error(err));
    }
}
