#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[tauri::command]
fn command_with_message(message: String) -> Result<String, String> {
  Ok(format!("hello {}", message))
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      command_with_message,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
