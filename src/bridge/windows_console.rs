// src/windows_console.rs
// Windows console attachment for MCP stdio transport
// 
// Problem: GUI applications (like Zed Editor) spawn processes without a console,
// causing GetStdHandle() to return INVALID_HANDLE_VALUE and stdio to fail.
// Solution: Attach to the parent process's console if we're running without one.

#[cfg(target_os = "windows")]
pub fn attach_console() {
    use windows_sys::Win32::Foundation::{INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Console::{
        AttachConsole, ATTACH_PARENT_PROCESS, GetStdHandle, STD_OUTPUT_HANDLE,
    };
    
    unsafe {
        // Check if we have a valid stdout handle
        let stdout: *mut std::ffi::c_void = GetStdHandle(STD_OUTPUT_HANDLE);
        
        // If stdout is invalid or null, we need to attach to parent console
        if stdout.is_null() || stdout as isize == INVALID_HANDLE_VALUE {
            // Try to attach to parent console
            let result = AttachConsole(ATTACH_PARENT_PROCESS);
            
            if result != 0 {
                eprintln!("[RoBoT] Attached to parent console for stdio transport");
                
                // Now reopen stdin/stdout/stderr from the console
                // This is handled by the Windows C runtime automatically once a console is attached
                // But tokio's async IO may need explicit handling
                
                // Force reopen the standard handles by touching them
                let _ = std::io::stdout();
                let _ = std::io::stderr();
            } else {
                // Failed to attach - likely running without a console at all
                // This is expected when running as a true Windows GUI application
                eprintln!("[RoBoT] Warning: No console available, stdio may not work");
            }
        } else {
            eprintln!("[RoBoT] Console already available");
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn attach_console() {
    // No-op on non-Windows platforms
}
