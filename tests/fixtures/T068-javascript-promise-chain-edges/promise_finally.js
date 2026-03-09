function request() {
    return api.call()
        .then(handleSuccess)
        .catch(handleError)
        .finally(cleanup);
}
