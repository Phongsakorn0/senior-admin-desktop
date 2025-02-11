use reqwest; // นำเข้า reqwest

#[tauri::command]
async fn notify_clients(message: i32) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:8080/api/sse/notify")
        .json(&serde_json::json!({ "message": message }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Server error: {}", res.status()))
    }
}

#[tauri::command]
async fn receive() -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:8080/api/sse/getupdatedata")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        Err(format!("Server error: {}", res.status()))
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![notify_clients, receive])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}