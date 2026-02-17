async fn process() -> Result<(), Error> {
    let data = client.fetch().await?;
    let result = save_data(data).await?;
    Ok(())
}
