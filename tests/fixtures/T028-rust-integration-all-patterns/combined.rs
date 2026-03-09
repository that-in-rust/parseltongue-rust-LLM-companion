use std::collections::HashMap;

async fn process_users(users: Vec<User>) -> Result<HashMap<String, String>, Error> {
    let names: Vec<String> = users
        .iter()
        .filter(|u| u.active)
        .map(|u| u.name.clone())
        .collect();

    let data = fetch_data().await?;
    let result = data.field_value;

    Ok(HashMap::new())
}
