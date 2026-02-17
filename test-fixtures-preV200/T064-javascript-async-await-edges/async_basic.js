async function loadData() {
    const response = await fetch('/api/data');
    const json = await response.json();
    await saveToDb(json);
}
