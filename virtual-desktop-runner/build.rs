fn main() {
    windows::build! {
        Windows::Win32::System::Threading::*,
        Windows::Win32::System::StationsAndDesktops::*,
        Windows::Win32::System::Pipes::CreatePipe,
        Windows::Win32::Foundation::*,
        Windows::Win32::UI::WindowsAndMessaging::DESKTOP_CREATEWINDOW,
        Windows::Win32::System::WindowsProgramming::INFINITE,
    };
}
