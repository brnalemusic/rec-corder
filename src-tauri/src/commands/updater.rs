use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Listener, Manager, Wry};
use tauri_plugin_updater::Update;
use tauri_plugin_updater::UpdaterExt;

pub struct PendingUpdate(pub Mutex<Option<Update>>);

#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle<Wry>,
    state: tauri::State<'_, PendingUpdate>
) -> Result<Option<(String, Option<String>)>, String> {
    if let Ok(updater) = app.updater() {
        if let Ok(Some(update)) = updater.check().await {
            let update_version = update.version.clone();
            let current_version = app.package_info().version.to_string();

            // Strict semver check
            let parsed_update = semver::Version::parse(&update_version);
            let parsed_current = semver::Version::parse(&current_version);

            if let (Ok(_u_ver), Ok(_c_ver)) = (parsed_update, parsed_current) {
                // Em produção (release), impede downgrades.
                // Em desenvolvimento (debug), permite ver a tela mesmo na mesma versão para testes.
                #[cfg(not(debug_assertions))]
                if _u_ver <= _c_ver {
                    return Ok(None);
                }
            }

            let mut body = update.body.clone();

            // Busca os Release Notes reais via API do GitHub usando o reqwest já disponível
            let client = reqwest::Client::builder()
                .user_agent("rec-corder-updater")
                .build();
            
            if let Ok(client) = client {
                if let Ok(response) = client.get("https://api.github.com/repos/brnalemusic/rec-corder/releases/latest").send().await {
                    if let Ok(text) = response.text().await {
                        if let Ok(release) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(github_body) = release.get("body").and_then(|b| b.as_str()) {
                                if !github_body.is_empty() {
                                    body = Some(github_body.to_string());
                                }
                            }
                        }
                    }
                }
            }

            *state.0.lock() = Some(update);
            return Ok(Some((update_version, body)));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn show_updater(app: AppHandle<Wry>, version: String, body: Option<String>) -> Result<(), String> {
    if let Some(updater_window) = app.get_webview_window("updater") {
        let _ = updater_window.show();
        let _ = updater_window.unminimize();
        let _ = updater_window.set_focus();
    } else {
        let url = "updater.html".to_string();
        let window = tauri::WebviewWindowBuilder::new(
            &app,
            "updater",
            tauri::WebviewUrl::App(url.into()),
        )
        .title("Atualização - Rec Corder")
        .inner_size(750.0, 800.0)
        .resizable(true)
        .decorations(true)
        .center()
        .always_on_top(true)
        .visible(false) // Inicia oculta
        .build()
        .map_err(|e| e.to_string())?;

        // Handshake: Espera o frontend avisar que está pronto para receber os dados
        let v = version.clone();
        let b = body.clone();
        let w_handle = window.clone();
        
        let w_handle_close = window.clone();
        window.listen("updater-close", move |_| {
            let _ = w_handle_close.close();
        });

        window.listen("updater-ready", move |_| {
            let _ = w_handle.emit("updater-data", (v.clone(), b.clone()));
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn get_release_notes(app: AppHandle<Wry>, version: String) -> Result<Option<String>, String> {
    let _ = version; // Mark as used to avoid warnings
    // 1. Tenta ler do diretório de recursos (se estiver empacotado)
    if let Ok(mut resource_path) = app.path().resource_dir() {
        resource_path.push("UPDATE.md");
        if let Ok(content) = std::fs::read_to_string(resource_path) {
            return Ok(Some(content));
        }
    }
    
    // 2. Tenta ler do diretório raiz durante o desenvolvimento (um nível acima de src-tauri)
    if let Ok(content) = std::fs::read_to_string("../UPDATE.md") {
        return Ok(Some(content));
    }

    // 3. Fallback para lowercase
    if let Ok(content) = std::fs::read_to_string("../update.md") {
        return Ok(Some(content));
    }

    // 4. Fallback final para o diretório atual
    if let Ok(content) = std::fs::read_to_string("UPDATE.md") {
        return Ok(Some(content));
    }

    Ok(None)
}

#[tauri::command]
pub async fn show_release_notes(app: AppHandle<Wry>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("release-notes") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    } else {
        let url = "release-notes.html".to_string();
        let window = tauri::WebviewWindowBuilder::new(
            &app,
            "release-notes",
            tauri::WebviewUrl::App(url.into()),
        )
        .title("Notas de Lançamento - Rec Corder")
        .inner_size(750.0, 800.0)
        .resizable(true)
        .decorations(true)
        .center()
        .always_on_top(true)
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;

        let w_handle_close = window.clone();
        window.listen("release-notes-close", move |_| {
            let _ = w_handle_close.close();
        });

        // A janela agora busca os dados sozinha via comandos quando carrega
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn install_update(
    app: AppHandle<Wry>,
    state: tauri::State<'_, PendingUpdate>
) -> Result<(), String> {
    let update = state.0.lock().take().ok_or("No pending update")?;
    let handle_clone = app.clone();
    
    tauri::async_runtime::spawn(async move {
        let handle_for_progress = handle_clone.clone();
        let handle_for_finish = handle_clone.clone();
        let handle_for_error = handle_clone.clone();
        
        match update.download_and_install(
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
        ).await {
            Ok(_) => {
                let _ = handle_clone.restart();
            },
            Err(e) => {
                let _ = handle_for_error.emit("update-error", e.to_string());
            }
        }
    });
    
    Ok(())
}

#[tauri::command]
pub fn open_link(app: AppHandle<Wry>, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url(url, None::<String>).map_err(|e| e.to_string())
}
