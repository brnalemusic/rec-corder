// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// Evita janela adicional de console no Windows durante release, NÃO REMOVA!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Ponto de entrada principal da aplicação em Rust/Tauri.
fn main() {
    rec_corder_lib::run()
}
