#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ashuk_core::{converter, format_meta};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use env_logger;

use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputResult {
    pub path: String,
    pub size: u64,
    pub writable_extentions: Vec<String>,
    pub extention: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileContext {
    pub status: converter::ConvertStatus,
    pub input: InputResult,
    pub output: Option<converter::CovertResult>,
}

type FileList = HashMap<String, FileContext>;

pub struct FileState {
    files: Mutex<FileList>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EmitFileOperation {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmitFileRequestBody {
    files: FileList,
    operation: EmitFileOperation,
}

impl FileState {
    pub fn new(files: FileList) -> Self {
        Self {
            files: Mutex::new(files),
        }
    }

    pub fn add_file(&self, file_path: &str) -> FileContext {
        let mut files = self.files.lock().unwrap();
        // init data
        let file = FileContext {
            status: converter::ConvertStatus::Initialized,
            input: InputResult {
                path: file_path.to_string().clone(),
                size: std::fs::metadata(&file_path)
                    .expect("There is no file.")
                    .len(),
                writable_extentions: format_meta::get_formats()
                    .iter()
                    .filter(|x| format_meta::ImageFormat::can_write(&x))
                    .map(move |y| <str as ToString>::to_string(&*y.extensions_str()[0]))
                    .collect::<Vec<String>>(),
                extention: format_meta::get_format_from_path(&file_path)
                    .unwrap()
                    .extensions_str()[0]
                    .to_string(),
            },
            output: Some(converter::CovertResult {
                size: 0,
                path: "".to_string(),
                elapsed: 0,
                extention: format_meta::get_format_from_path(&file_path)
                    .unwrap()
                    .extensions_str()[0]
                    .to_string(),
            }),
        };
        // update hashmap
        files.entry(file_path.to_string()).or_insert(file.clone());

        file
    }

    pub fn update_file(&self, path: String, file_status: FileContext) -> FileContext {
        let mut files = self.files.lock().unwrap();
        let new_file = FileContext {
            status: file_status.status,
            input: file_status.input,
            output: file_status.output,
        };
        // update hashmap
        files.entry(path).or_insert(new_file.clone());

        new_file
    }

    pub fn delete_file(&self, path: String) {
        let mut files = self.files.lock().unwrap();
        let file = files.get(&path);
        let deleted = file.clone();
        // update hashmap
        files.retain(|x, _| x == &path);
    }

    pub fn get_files(&self) -> FileList {
        let files = self.files.lock().unwrap();
        files.clone()
    }

    pub fn clear(&self) {
        let mut files = self.files.lock().unwrap();
        files.clear();
    }

    pub fn process_convert(&self, app_handle: &tauri::AppHandle, emitter_name: &str, path: std::string::String, file: FileContext) {
        // update state
        let updated_file = self.update_file(
            path.clone(),
            FileContext {
                status: converter::ConvertStatus::Pending,
                input: file.clone().input,
                output: None,
            },
        );
        // notify to client for start converting
        notify_file_to_client(&app_handle, &updated_file, emitter_name);

        // convert image
        let result = converter::covert_to_target_extention(
            &path,
            &file.output.unwrap().extention,
            75.0,
        );

        if result.is_ok() {
            // update state
            let updated_file = self.update_file(
                path,
                FileContext {
                    status: converter::ConvertStatus::Success,
                    input: file.input,
                    output: Some(result.unwrap()),
                },
            );
            // notify to client for success
            notify_file_to_client(&app_handle, &updated_file, emitter_name);
        } else {
            // update state
            let updated_file = self.update_file(
                path,
                FileContext {
                    status: converter::ConvertStatus::Failed,
                    input: file.input,
                    output: None,
                },
            );
            // notify to client for failure
            notify_file_to_client(&app_handle, &updated_file, emitter_name);
        }
    }
}

fn notify_file_to_client(
    app_handle: &tauri::AppHandle,
    file_context: &FileContext,
    emitter_name: &str,
) {
    let serialized = serde_json::to_string(&file_context).expect("Invalid data format");
    app_handle.emit_all(&emitter_name, serialized).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SupportedFormatMeta {
    ext: String,
    readable: bool,
    writable: bool,
}

#[tauri::command]
fn get_supported_extentions() -> Result<Vec<SupportedFormatMeta>, String> {
    let extensions = format_meta::get_formats()
        .iter()
        .map(|x| {
            x.extensions_str().iter().map(move |y|
                // read/write flag decided by ImageFormat that extended 'image' crate
                SupportedFormatMeta {
                    ext: <str as ToString>::to_string(*y),
                    readable: format_meta::ImageFormat::can_read(&x),
                    writable: format_meta::ImageFormat::can_write(&x)
                })
        })
        .flatten()
        .collect::<Vec<SupportedFormatMeta>>();
    Ok(extensions)
}

fn convert_file_handler(app: &tauri::AppHandle) {
    let emitter_name = "listen-file";
    let file_state = FileState::new(HashMap::new());
    let app_handle = app.app_handle();
    // listen file input
    let emit_file_create = app_handle
        .app_handle()
        .listen_global("emit-file", move |event| {
            // get task
            let task: EmitFileRequestBody = serde_json::from_str(&event.payload().unwrap())
                .expect("JSON was not well-formatted");

            match task.operation {
                // notify to client for ready
                EmitFileOperation::Create => {
                    // process parallelly
                    task.files.into_par_iter().for_each(|(path, _)| {
                        let add_file = file_state.add_file(&path);
                        notify_file_to_client(&app_handle, &add_file, emitter_name);
                    });
                }
                // convert
                EmitFileOperation::Update => {
                    // wheather process parallelly or not
                    let paralell_processable = task.clone().files.into_par_iter().all(|(path, _)|
                        // process strategy with multithreading depends on it's library implementation
                        matches!(
                            format_meta::ImageFormat::process_strategy(&format_meta::get_format_from_path(&path).unwrap()),
                            format_meta::ProcessStrategy::Parallel
                        )
                    );

                    match paralell_processable {
                        true => {
                            // process parallelly
                            task.files
                                .into_par_iter()
                                // skip converted file
                                .filter(|(_, file)| {
                                    !matches!(file.status, converter::ConvertStatus::Success)
                                })
                                .for_each(|(path, file)| file_state.process_convert(&app_handle, &emitter_name, path, file));
                        }
                        _ => {
                            // process seriesly
                            task.files
                                .into_iter()
                                // skip converted file
                                .filter(|(_, file)| {
                                    !matches!(file.status, converter::ConvertStatus::Success)
                                })
                                .for_each(|(path, file)| file_state.process_convert(&app_handle, &emitter_name, path, file));
                        },
                    };
                }
                _ => {
                    unimplemented!()
                }
            }
        });
}

#[cfg(debug_assertions)]
fn init_logger() {
    env::set_var("RUST_LOG", "debug");
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let ts = buf.timestamp();
            writeln!(
                buf,
                "[{} {} {}] {} {}:{}",
                ts,
                record.level(),
                record.target(),
                record.args(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
            )
        })
        .init();

}

fn main() {
    init_logger();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_supported_extentions,])
        .setup(|app| {
            convert_file_handler(&app.app_handle());

            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
