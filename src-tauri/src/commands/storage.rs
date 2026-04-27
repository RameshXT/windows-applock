use crate::secure_storage;
use tauri::AppHandle;

#[tauri::command]
pub async fn verify_storage_integrity(app_handle: AppHandle) -> Result<bool, String> {
    secure_storage::verify_storage_integrity_internal(app_handle).await
}

#[tauri::command]
pub async fn get_storage_status(
    app_handle: AppHandle,
) -> Result<secure_storage::StorageStatus, String> {
    secure_storage::get_storage_status_internal(app_handle).await
}
