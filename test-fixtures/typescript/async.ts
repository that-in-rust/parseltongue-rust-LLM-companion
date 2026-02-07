// Test fixture for TypeScript async/await detection
// REQ-TYPESCRIPT-004.0

class DataFetcher {
    async load() {
        // Await expressions
        const data = await fetchData();
        const user = await getUser();
        const settings = await this.loadSettings();
    }

    async processWithPromises() {
        // Promise chains
        fetchData()
            .then(handleSuccess)
            .catch(handleError)
            .finally(cleanup);
    }

    private async loadSettings() {
        return { theme: 'dark' };
    }
}

async function fetchData(): Promise<any> {
    return { id: 1 };
}

async function getUser(): Promise<any> {
    return { name: 'Alice' };
}

function handleSuccess(data: any) {}
function handleError(error: any) {}
function cleanup() {}
