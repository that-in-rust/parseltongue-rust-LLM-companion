class Fetcher {
    async load() {
        const data = await fetchData();
        const user = await getUser();
    }
}
