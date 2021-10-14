use std::mem::{self, MaybeUninit};
use std::{env, ptr};

use defer_lite::defer;
use uuid::Uuid;
use widestring::U16CString;

windows::include_bindings!();
use Windows::Win32::{
    Foundation::*,
    Security::SECURITY_ATTRIBUTES,
    System::{Pipes::*, StationsAndDesktops::*, Threading::*, WindowsProgramming::INFINITE},
    UI::WindowsAndMessaging::DESKTOP_CREATEWINDOW,
};

const WAIT_OBJECT_1: WAIT_RETURN_CAUSE = WAIT_RETURN_CAUSE(WAIT_OBJECT_0.0 + 1);
const STILL_ACTIVE: NTSTATUS = STATUS_PENDING;
const TRUE: BOOL = BOOL(1);
const FALSE: BOOL = BOOL(0);

fn main() {
    let mut args = env::args();
    // skip program name
    args.next().expect("Invalid command line provided.");

    let target_path = args.next().expect("Missing target app path.");
    while let Some(arg) = args.next() {
        if arg == "--" {
            break;
        }
    }
    let target_args = args;

    let desktop_name =
        U16CString::from_str(format!("virtual-desktop-runner/{}", Uuid::new_v4())).unwrap();
    let desktop_name_ptr = desktop_name.as_ptr() as *mut _;
    let desktop_handle = unsafe {
        CreateDesktopW(
            PWSTR(desktop_name_ptr),
            PWSTR::default(),
            ptr::null_mut(),
            0,
            DESKTOP_CREATEWINDOW as _,
            ptr::null_mut(),
        )
    };
    if desktop_handle.0 == 0 {
        panic!(
            "Failed to create virtual desktop: {:#?}",
            windows::Error::from_win32()
        );
    }
    defer! {
        unsafe { CloseDesktop(desktop_handle) }.ok().expect("Failed to close virtual desktop.")
    }

    let mut command_line = snailquote::escape(&target_path)
        .as_ref()
        .trim_matches('\"')
        .to_string();
    for arg in target_args {
        let escaped_arg = snailquote::escape(&arg);
        command_line.push(' ');
        command_line.push_str(&escaped_arg);
    }

    // println!("Command line: {}", command_line);
    // println!("Target Path: {}", target_path);

    let wide_command_line =
        U16CString::from_str(&command_line).expect("Failed to convert command line to widestring.");
    let wide_target_path = U16CString::from_str(&target_path)
        .expect("Failed to convert target app path to widestring.");

    let mut stdin_read = MaybeUninit::uninit();
    let mut stdin_write = MaybeUninit::uninit();
    let mut stdout_read = MaybeUninit::uninit();
    let mut stdout_write = MaybeUninit::uninit();
    let mut stderr_read = MaybeUninit::uninit();
    let mut stderr_write = MaybeUninit::uninit();

    let security_attributes = SECURITY_ATTRIBUTES {
        nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as _,
        bInheritHandle: TRUE,
        lpSecurityDescriptor: ptr::null_mut(),
    };

    unsafe {
        CreatePipe(
            stdin_read.as_mut_ptr(),
            stdin_write.as_mut_ptr(),
            &security_attributes,
            0,
        )
    }
    .ok()
    .expect("Failed to create piped stdin");
    let stdin_read = unsafe { stdin_read.assume_init() };
    let stdin_write = unsafe { stdin_write.assume_init() };
    defer! {
        unsafe { CloseHandle(stdin_read) }.ok().expect("Failed to close piped stdin handle.");
        unsafe { CloseHandle(stdin_write) }.ok().expect("Failed to close piped stdin handle.");
    }
    unsafe {
        CreatePipe(
            stdout_read.as_mut_ptr(),
            stdout_write.as_mut_ptr(),
            &security_attributes,
            0,
        )
    }
    .ok()
    .expect("Failed to create piped stdout");
    let stdout_read = unsafe { stdout_read.assume_init() };
    let stdout_write = unsafe { stdout_write.assume_init() };
    defer! {
        unsafe { CloseHandle(stdout_read) }.ok().expect("Failed to close piped stdout handle.");
        unsafe { CloseHandle(stdout_write) }.ok().expect("Failed to close piped stdout handle.");
    }
    unsafe {
        CreatePipe(
            stderr_read.as_mut_ptr(),
            stderr_write.as_mut_ptr(),
            &security_attributes,
            0,
        )
    }
    .ok()
    .expect("Failed to create piped stderr");
    let stderr_read = unsafe { stderr_read.assume_init() };
    let stderr_write = unsafe { stderr_write.assume_init() };
    defer! {
        unsafe { CloseHandle(stderr_read) }.ok().expect("Failed to close piped stderr handle.");
        unsafe { CloseHandle(stderr_write) }.ok().expect("Failed to close piped stderr handle.");
    }

    unsafe { SetHandleInformation(stdin_write, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }
        .ok()
        .expect("Failed to make stdin handle uninheritable.");
    unsafe { SetHandleInformation(stdout_read, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }
        .ok()
        .expect("Failed to make stdout handle uninheritable.");
    unsafe { SetHandleInformation(stderr_read, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }
        .ok()
        .expect("Failed to make stderr handle uninheritable.");

    let startup_info = STARTUPINFOW {
        cb: mem::size_of::<STARTUPINFOW>() as _,
        lpReserved: PWSTR::default(),
        lpDesktop: PWSTR(desktop_name_ptr),
        lpTitle: PWSTR::default(),
        dwX: 0,
        dwY: 0,
        dwXSize: 0,
        dwYSize: 0,
        dwXCountChars: 0,
        dwYCountChars: 0,
        dwFillAttribute: 0,
        dwFlags: STARTF_USESTDHANDLES,
        wShowWindow: 0,
        cbReserved2: 0,
        lpReserved2: ptr::null_mut(),
        hStdInput: stdin_read,
        hStdOutput: stdout_write,
        hStdError: stderr_write,
    };

    let mut process_info = MaybeUninit::uninit();
    unsafe {
        CreateProcessW(
            PWSTR(wide_target_path.as_ptr() as *mut _),
            PWSTR(wide_command_line.as_ptr() as *mut _),
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            PROCESS_CREATION_FLAGS::default(),
            ptr::null_mut(),
            PWSTR(ptr::null_mut()),
            &startup_info,
            process_info.as_mut_ptr(),
        )
    }
    .ok()
    .expect("Failed to start target application.");

    let process_info = unsafe { process_info.assume_init() };

    unsafe { CloseHandle(process_info.hThread) }
        .ok()
        .expect("Failed to close target thread initial thread handle.");

    defer! {
        unsafe { CloseHandle(process_info.hProcess) }.ok().expect("Failed to close target application handle.");
    }

    let cancel_event = unsafe { CreateEventW(ptr::null_mut(), TRUE, FALSE, PWSTR::default()) };

    ctrlc::set_handler(move || {
        unsafe { SetEvent(cancel_event) }
            .ok()
            .expect("Failed to abort wait for target app exit.")
    })
    .expect("Failed to set Ctrl-C handler.");

    let wait_result = unsafe {
        WaitForMultipleObjects(
            2,
            [process_info.hProcess, cancel_event].as_ptr(),
            FALSE,
            INFINITE,
        )
    };
    match wait_result {
        WAIT_OBJECT_0 | WAIT_OBJECT_1 => (),
        WAIT_ABANDONED => unreachable!(),
        WAIT_TIMEOUT => unreachable!(),
        WAIT_FAILED => panic!(
            "Failed to wait for target app exit: {:#?}",
            windows::Error::from_win32()
        ),
        _ => unreachable!(),
    }

    let mut exit_code = MaybeUninit::uninit();
    unsafe { GetExitCodeProcess(process_info.hProcess, exit_code.as_mut_ptr()) }.ok().expect("Failed to get target application exit code.");
    let exit_code = unsafe { exit_code.assume_init() };
    if exit_code == STILL_ACTIVE.0 {
        unsafe { TerminateProcess(process_info.hProcess, 0) }.ok().expect("Failed to terminate target application.");
    }
}
