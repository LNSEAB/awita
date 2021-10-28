use windows::Win32::{
    Foundation::*,
    UI::{Controls::*, WindowsAndMessaging::*},
};
use std::path::{Path, PathBuf};

fn make_int_resource(id: u16) -> PWSTR {
    PWSTR(id as _)
}

#[derive(Clone, Debug)]
pub enum Icon {
    Resource(u16),
    File(PathBuf),
}

impl Icon {
    fn load_impl(&self, cx: i32, cy: i32) -> windows::runtime::Result<HICON> {
        unsafe {
            let icon = match self {
                Icon::Resource(id) => LoadImageW(
                    HINSTANCE::default(),
                    make_int_resource(*id),
                    IMAGE_ICON,
                    cx,
                    cy,
                    LR_SHARED,
                ),
                Icon::File(path) => LoadImageW(
                    HINSTANCE::default(),
                    path.to_string_lossy().as_ref(),
                    IMAGE_ICON,
                    cx,
                    cy,
                    LR_SHARED | LR_LOADFROMFILE,
                ),
            };
            if icon == HANDLE::default() {
                return Err(windows::runtime::Error::from_win32());
            }
            Ok(HICON(icon.0))
        }
    }

    pub(crate) fn load(&self) -> windows::runtime::Result<HICON> {
        unsafe { self.load_impl(GetSystemMetrics(SM_CXICON), GetSystemMetrics(SM_CYICON)) }
    }

    pub(crate) fn load_small(&self) -> windows::runtime::Result<HICON> {
        unsafe { self.load_impl(GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON)) }
    }
}

impl<'a> From<&'a Path> for Icon {
    fn from(src: &'a Path) -> Self {
        Icon::File(src.into())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cursor {
    AppStarting,
    Arrow,
    Cross,
    Hand,
    Help,
    IBeam,
    No,
    SizeAll,
    SizeNESW,
    SizeNS,
    SizeNWSE,
    SizeWE,
    SizeUpArrow,
    Wait,
}

impl Cursor {
    pub(crate) fn name(&self) -> PWSTR {
        match self {
            Self::AppStarting => IDC_APPSTARTING,
            Self::Arrow => IDC_ARROW,
            Self::Cross => IDC_CROSS,
            Self::Hand => IDC_HAND,
            Self::Help => IDC_HELP,
            Self::IBeam => IDC_IBEAM,
            Self::No => IDC_NO,
            Self::SizeAll => IDC_SIZEALL,
            Self::SizeNESW => IDC_SIZENESW,
            Self::SizeNS => IDC_SIZENS,
            Self::SizeNWSE => IDC_SIZENWSE,
            Self::SizeWE => IDC_SIZEWE,
            Self::SizeUpArrow => IDC_UPARROW,
            Self::Wait => IDC_WAIT,
        }
    }

    pub(crate) fn set(&self) {
        unsafe {
            SetCursor(LoadCursorW(HINSTANCE::default(), self.name()));
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::Arrow
    }
}
