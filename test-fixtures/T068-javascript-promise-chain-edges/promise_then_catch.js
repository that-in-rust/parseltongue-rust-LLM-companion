function fetchUser(id) {
    return fetch('/api/user/' + id)
        .then(res => res.json())
        .catch(err => console.error(err));
}
