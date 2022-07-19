#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use ashuk_core::converter;
use futures::future;
use tokio;

#[tauri::command]
async fn command_covert_to_webp(file_path: Vec<String>) -> Result<Vec<converter::CovertResult>, String> {
  let tasks: Vec<_> = file_path
    .iter()
    .map(|path| async {
      let path = path.clone();
      tokio::spawn(async move {
        let res = converter::covert_to_webp(&path, 100.0).await;
        if res.is_ok() {
          res.unwrap()
        } else {
          converter::CovertResult {
            status: converter::ConvertStatus::Failed,
            input: None,
            output: None,
            elapsed: None,
          }
        }
      }).await.unwrap()
    })
    .collect();

  let result = future::join_all(tasks).await;

  Ok(result)
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      command_covert_to_webp,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
