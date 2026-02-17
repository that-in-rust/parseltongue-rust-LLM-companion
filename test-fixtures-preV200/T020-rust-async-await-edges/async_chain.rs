async fn complex_operation() {
    let value = api.fetch_user().await.process().await;
}
