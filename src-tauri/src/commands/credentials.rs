    use tauri::AppHandle;
use crate::credential_manager;

#[tauri::command]
pub async fn set_credential(app_handle: AppHandle, pin_or_password: String, cred_type: String) -> Result<(), String> {
    credential_manager::set_credential_internal(&app_handle, pin_or_password, cred_type)
}


#[tauri::command]
pub async fn update_credential(
    app_handle: AppHandle,
    old_input: String,
    new_input: String,
    cred_type: String,
) -> Result<(), String> {
    credential_manager::update_credential_internal(&app_handle, old_input, new_input, cred_type)
}

#[tauri::command]
pub async fn get_credential_type(app_handle: AppHandle) -> Result<String, String> {
    credential_manager::get_credential_type_internal(&app_handle)
}

#[tauri::command]
pub async fn check_rehash_needed() -> bool {
    credential_manager::get_rehash_needed()
}
