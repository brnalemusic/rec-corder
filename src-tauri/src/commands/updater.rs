use parking_lot::Mutex;
use tauri::Emitter;
use tauri_plugin_updater::Update;
use tauri_plugin_updater::UpdaterExt;

pub struct PendingUpdate(pub Mutex<Option<Update>>);

#[tauri::command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
    state: tauri::State<'_, PendingUpdate>
) -> Result<Option<String>, String> {
    if let Ok(updater) = app.updater() {
        if let Ok(Some(update)) = updater.check().await {
            let version = update.version.clone();
            *state.0.lock() = Some(update);
            return Ok(Some(version));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, PendingUpdate>
) -> Result<(), String> {
    let update = state.0.lock().take().ok_or("No pending update")?;
    let handle_clone = app.clone();
    
    tauri::async_runtime::spawn(async move {
        let handle_for_progress = handle_clone.clone();
        let handle_for_finish = handle_clone.clone();
        
        let _ = update.download_and_install(
            move |chunk_length, content_length| {
                if let Some(total) = content_length {
                    let mut payload = std::collections::HashMap::new();
                    payload.insert("chunk", chunk_length as u64);
                    payload.insert("total", total);
                    let _ = handle_for_progress.emit("update-progress", payload);
                }
            },
            move || {
                let _ = handle_for_finish.emit("update-finished", ());
            }
        ).await;
        
        handle_clone.restart();
    });
    
    Ok(())
}
