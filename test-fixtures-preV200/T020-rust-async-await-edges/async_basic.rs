async fn load_data() {
    let result = fetch_data().await;
    let json = client.get().await;
}
