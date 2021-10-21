use std::os::windows::io::AsRawHandle;
use std::{
    env,
    mem::{self, MaybeUninit},
    ptr,
};

use defer_lite::defer;
use uuid::Uuid;
use widestring::U16CString;

windows::include_bindings!();
use Windows::Win32::{
    Foundation::*,
    System::{StationsAndDesktops::*, Threading::*, WindowsProgramming::INFINITE},
    UI::WindowsAndMessaging::DESKTOP_CREATEWINDOW,
};

const WAIT_OBJECT_1: WAIT_RETURN_CAUSE = WAIT_RETURN_CAUSE(WAIT_OBJECT_0.0 + 1);
const STILL_ACTIVE: NTSTATUS = STATUS_PENDING;
const TRUE: BOOL = BOOL(1);
const FALSE: BOOL = BOOL(0);

fn run(mut args: impl Iterator<Item = String>) -> i32 {
    // skip program name
    args.next().expect("Invalid command line provided.");

    let target_path = args.next().expect("Missing target app path.");
    for arg in args.by_ref() {
        if arg == "--" {
            break;
        }
    }
    let target_args = args;

    let mut desktop_name =
        U16CString::from_str(format!("virtual-desktop-runner/{}", Uuid::new_v4())).unwrap();
    let desktop_handle = unsafe {
        CreateDesktopW(
            PWSTR(desktop_name.as_mut_ptr()),
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

    let mut command_line = snailquote::escape(&target_path).to_string();
    for arg in target_args {
        let escaped_arg = snailquote::escape(&arg);
        command_line.push(' ');
        command_line.push_str(&escaped_arg);
    }

    let mut wide_command_line =
        U16CString::from_str(&command_line).expect("Failed to convert command line to widestring.");

    let startup_info = STARTUPINFOW {
        cb: mem::size_of::<STARTUPINFOW>() as _,
        lpReserved: PWSTR::default(),
        lpDesktop: PWSTR(unsafe { desktop_name.as_mut_ptr() }),
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
        hStdInput: HANDLE(std::io::stdin().as_raw_handle() as isize),
        hStdOutput: HANDLE(std::io::stdout().as_raw_handle() as isize),
        hStdError: HANDLE(std::io::stderr().as_raw_handle() as isize),
    };

    let mut process_info = MaybeUninit::uninit();
    unsafe {
        CreateProcessW(
            PWSTR::default(),
            PWSTR(wide_command_line.as_mut_ptr()),
            ptr::null_mut(),
            ptr::null_mut(),
            true,
            CREATE_UNICODE_ENVIRONMENT,
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
        .expect("Failed to close initial thread handle of target.");

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
    unsafe { GetExitCodeProcess(process_info.hProcess, exit_code.as_mut_ptr()) }
        .ok()
        .expect("Failed to get target application exit code.");
    let mut exit_code = unsafe { exit_code.assume_init() };
    if exit_code == STILL_ACTIVE.0 {
        unsafe { TerminateProcess(process_info.hProcess, 0) }
            .ok()
            .expect("Failed to terminate target application.");
        exit_code = 0;
    }

    exit_code as i32
}

fn main() {
    let exit_code = run(env::args());
    std::process::exit(exit_code);
}
