class Fetcher {
    async load() {
        const data = await this.fetchData();
        return await this.processData(data);
    }
}
