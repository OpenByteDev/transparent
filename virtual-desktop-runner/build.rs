fn main() {
    windows::build! {
        Windows::Win32::{
            Security::SECURITY_ATTRIBUTES,
            UI::{
                DisplayDevices::DEVMODEW,
                WindowsAndMessaging::DESKTOP_CREATEWINDOW
            },
            System::{
                Threading::{
                    PROCESS_CREATION_FLAGS,
                    // WAIT_OBJECT_0,
                    WAIT_RETURN_CAUSE,
                    STARTUPINFOW,
                    // STARTF_USESTDHANDLES,
                    CreateProcessW,
                    PROCESS_INFORMATION,
                    PROCESS_CREATION_FLAGS,
                    CreateEventW,
                    SetEvent,
                    WaitForMultipleObjects,
                    GetExitCodeProcess,
                    TerminateProcess
                },
                WindowsProgramming::INFINITE,
                Pipes::CreatePipe,
                StationsAndDesktops::{
                    HDESK,
                    CreateDesktopW,
                    CloseDesktop,
                }
            },
        },
        Windows::Win32::Foundation::{
            NTSTATUS,
            CloseHandle,
            SetHandleInformation,
            STATUS_PENDING
        },
    };
}
