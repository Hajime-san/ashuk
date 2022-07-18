#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use ashuk_core::{ covert_to_webp };
use futures::future;
use tokio;

#[tauri::command]
async fn command_covert_to_webp(file_path: Vec<String>) -> Result<String, String> {
  let tasks: Vec<_> = file_path
    .iter()
    .map(|path| {
      let path = path.clone();
      tokio::spawn(async move {
        let result = covert_to_webp(&path, 100.0).await;
      })
    })
    .collect();
  future::join_all(tasks).await;

  Ok("SUCCESS".to_string())
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      command_covert_to_webp,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
