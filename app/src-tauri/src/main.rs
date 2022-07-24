#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ashuk_core::{ converter, format_meta };
use futures::future;
use futures::stream::{self, StreamExt};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tokio;

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputResult {
    pub size: u64,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileContext {
    pub id: usize,
    pub status: converter::ConvertStatus,
    pub input: InputResult,
    pub output: Option<converter::CovertResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileStatus {
    pub status: converter::ConvertStatus,
    pub input: InputResult,
    pub output: Option<converter::CovertResult>,
}

type FileList = HashMap<usize, FileContext>;

pub struct FileState {
    files: Mutex<FileList>,
}

impl FileState {
    pub fn new(files: FileList) -> Self {
        Self {
            files: Mutex::new(files),
        }
    }

    pub fn add_file(&self, file_path: &str) -> FileContext {
        let mut files = self.files.lock().unwrap();
        // increment id
        let id = files.len() + 1;
        // init data
        let file = FileContext {
            id: id,
            status: converter::ConvertStatus::Initialized,
            input: InputResult {
                size: std::fs::metadata(&file_path)
                    .expect("There is no file.")
                    .len(),
                path: file_path.to_string().clone(),
            },
            output: None,
        };
        // update hashmap
        files.insert(id, file.clone());

        file
    }

    pub fn update_file(&self, id: usize, file_status: FileStatus) -> FileContext {
        let mut files = self.files.lock().unwrap();
        let file = files.get(&id);
        let new_file = FileContext {
            id: id,
            status: file_status.status,
            input: file_status.input,
            output: file_status.output,
        };
        // update hashmap
        files.entry(id).or_insert(new_file.clone());

        new_file
    }

    pub fn delete_file(&self, id: usize) {
        let mut files = self.files.lock().unwrap();
        let file = files.get(&id);
        let deleted = file.clone();
        // update hashmap
        files.retain(|&x, _| x == id);
    }

    pub fn get_files(&self) -> FileList {
        let files = self.files.lock().unwrap();
        files.clone()
    }

    pub fn clear(&self) {
        let mut files = self.files.lock().unwrap();
        files.clear();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SupportedFormatMeta {
    ext: String,
    readable: bool,
    writable: bool
}

#[tauri::command]
fn get_supported_extentions() -> Result<Vec<SupportedFormatMeta>, String> {
    let extensions = format_meta::get_formats()
        .iter()
        .map(|x|
                x.extensions_str()
                .iter()
                .map(move |y|
                    // read/write flag decided by ImageFormat that extended 'image' crate
                    SupportedFormatMeta {
                        ext: <str as ToString>::to_string(*y),
                        readable: format_meta::ImageFormat::can_read(&x),
                        writable: format_meta::ImageFormat::can_write(&x)
                    }
                ))
                .flatten()
                .collect::<Vec<SupportedFormatMeta>>();
    Ok(extensions)
}

fn convert_file_handler(app: &tauri::AppHandle) {
    let emitter_name = "listen-file";
    let file_state = FileState::new(HashMap::new());
    let app_handle = app.app_handle();
    // listen file input
    let id = app_handle
        .app_handle()
        .listen_global("emit-file", move |event| {
            // file path list to convert
            let task_file: Vec<String> = serde_json::from_str(&event.payload().unwrap())
                .expect("JSON was not well-formatted");

            task_file.into_par_iter().for_each(|path| {
                // notify to client for start
                let add_file = file_state.add_file(&path);
                let serialized = serde_json::to_string(&add_file).expect("Invalid data format");
                app_handle.emit_all(&emitter_name, serialized).unwrap();

                let result = converter::covert_to_webp(&path, 100.0);
                if result.is_ok() {
                    // update state
                    let updated_file = file_state.update_file(
                        add_file.id,
                        FileStatus {
                            status: converter::ConvertStatus::Success,
                            input: add_file.input,
                            output: Some(result.unwrap()),
                        },
                    );
                    // notify to client for success
                    let serialized =
                        serde_json::to_string(&updated_file).expect("Invalid data format");
                    app_handle.emit_all(&emitter_name, serialized).unwrap();
                } else {
                    // update state
                    let updated_file = file_state.update_file(
                        add_file.id,
                        FileStatus {
                            status: converter::ConvertStatus::Failed,
                            input: add_file.input,
                            output: None,
                        },
                    );
                    // notify to client for failure
                    let serialized =
                        serde_json::to_string(&updated_file).expect("Invalid data format");
                    app_handle.emit_all(&emitter_name, serialized).unwrap();
                }
            });
        });
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_supported_extentions,
        ])
        .setup(|app| {
            convert_file_handler(&app.app_handle());

            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
