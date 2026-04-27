use windows_sys::Win32::Foundation::{CloseHandle, FALSE, INVALID_HANDLE_VALUE, MAX_PATH};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, Thread32First, Thread32Next,
    PROCESSENTRY32, TH32CS_SNAPPROCESS, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::ProcessStatus::K32GetModuleFileNameExA;
use windows_sys::Win32::System::Threading::{
    OpenProcess, OpenThread, ResumeThread, SuspendThread, TerminateProcess,
    PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ, THREAD_QUERY_INFORMATION,
    THREAD_SUSPEND_RESUME,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub fn suspend_process(pid: u32) -> Result<(), String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err("Failed to create snapshot".to_string());
        }

        let mut te = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            cntUsage: 0,
            th32ThreadID: 0,
            th32OwnerProcessID: 0,
            tpBasePri: 0,
            tpDeltaPri: 0,
            dwFlags: 0,
        };

        if Thread32First(snapshot, &mut te) != FALSE {
            loop {
                if te.th32OwnerProcessID == pid {
                    let thread_handle = OpenThread(
                        THREAD_SUSPEND_RESUME | THREAD_QUERY_INFORMATION,
                        FALSE,
                        te.th32ThreadID,
                    );
                    if thread_handle != std::ptr::null_mut() {
                        SuspendThread(thread_handle);
                        CloseHandle(thread_handle);
                    }
                }
                if Thread32Next(snapshot, &mut te) == FALSE {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        Ok(())
    }
}

pub fn resume_process(pid: u32) -> Result<(), String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err("Failed to create snapshot".to_string());
        }

        let mut te = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            cntUsage: 0,
            th32ThreadID: 0,
            th32OwnerProcessID: 0,
            tpBasePri: 0,
            tpDeltaPri: 0,
            dwFlags: 0,
        };

        if Thread32First(snapshot, &mut te) != FALSE {
            loop {
                if te.th32OwnerProcessID == pid {
                    let thread_handle = OpenThread(
                        THREAD_SUSPEND_RESUME | THREAD_QUERY_INFORMATION,
                        FALSE,
                        te.th32ThreadID,
                    );
                    if thread_handle != std::ptr::null_mut() {
                        ResumeThread(thread_handle);
                        CloseHandle(thread_handle);
                    }
                }
                if Thread32Next(snapshot, &mut te) == FALSE {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        Ok(())
    }
}

pub fn kill_process(pid: u32) -> Result<(), String> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, FALSE, pid);
        if handle != std::ptr::null_mut() {
            TerminateProcess(handle, 1);
            CloseHandle(handle);
            Ok(())
        } else {
            Err("Failed to open process for termination".to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: String,
}

pub fn get_foreground_process_id() -> u32 {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return 0;
        }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        pid
    }
}

pub fn get_processes() -> Vec<ProcessInfo> {
    let mut processes = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return processes;
        }

        let mut pe = PROCESSENTRY32 {
            dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
            cntUsage: 0,
            th32ProcessID: 0,
            th32DefaultHeapID: 0,
            th32ModuleID: 0,
            cntThreads: 0,
            th32ParentProcessID: 0,
            pcPriClassBase: 0,
            dwFlags: 0,
            szExeFile: [0; 260],
        };

        if Process32First(snapshot, &mut pe) != FALSE {
            loop {
                let pid = pe.th32ProcessID;
                let name = pe
                    .szExeFile
                    .iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u8 as char)
                    .collect::<String>();

                let mut path = String::new();
                let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid);
                if handle != std::ptr::null_mut() {
                    let mut buffer = [0u8; MAX_PATH as usize];
                    let size = K32GetModuleFileNameExA(
                        handle,
                        std::ptr::null_mut(),
                        buffer.as_mut_ptr(),
                        MAX_PATH,
                    );
                    if size > 0 {
                        path = String::from_utf8_lossy(&buffer[..size as usize]).to_string();
                    }
                    CloseHandle(handle);
                }

                processes.push(ProcessInfo { pid, name, path });

                if Process32Next(snapshot, &mut pe) == FALSE {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
    }
    processes
}
