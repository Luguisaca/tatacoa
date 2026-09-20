#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::str::FromStr;
use tatacoa_app_api::{AppService, NewWorkRequest, RunRequest, WorkContext, WorkSummary};
use tatacoa_core::{Engagement, EngagementId};

type CommandResult<T> = Result<T, String>;

#[tauri::command]
fn list_work(workspace: String) -> CommandResult<Vec<Engagement>> {
    AppService::open(workspace)
        .list_work()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_work(workspace: String, request: NewWorkRequest) -> CommandResult<WorkContext> {
    AppService::open(workspace)
        .create_work(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn summarize(workspace: String, engagement_id: String) -> CommandResult<WorkSummary> {
    let id = EngagementId::from_str(&engagement_id).map_err(|error| error.to_string())?;
    AppService::open(workspace)
        .summarize(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn run_tool(workspace: String, request: RunRequest) -> CommandResult<WorkSummary> {
    let service = AppService::open(&workspace);
    let engagement_id = request.engagement_id.clone();
    service.run(request).map_err(|error| error.to_string())?;
    service
        .summarize(&engagement_id)
        .map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_work,
            create_work,
            summarize,
            run_tool
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| eprintln!("TATACOA desktop failed: {error}"));
}
