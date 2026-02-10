class AsyncProcessor {
    process() {
        fetchData()
            .then(handleSuccess)
            .catch(handleError);
    }
}
