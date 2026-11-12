//! The state of a Windows pen that winit leaves out: whether the eraser end is down (or the pen is
//! turned over) and whether the barrel button is held. winit hands a pen to egui as a touch whose
//! id is the Windows pointer id, so the pen's flags can be asked for with that id.
#![deny(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

/// `PEN_FLAG_BARREL`: the barrel button is pressed.
pub const PEN_FLAG_BARREL: u32 = 0x1;
/// `PEN_FLAG_INVERTED`: the pen is turned over (eraser end towards the screen).
pub const PEN_FLAG_INVERTED: u32 = 0x2;
/// `PEN_FLAG_ERASER`: the eraser button (or end) is pressed.
pub const PEN_FLAG_ERASER: u32 = 0x4;

/// What Windows reports for one pen pointer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PenState {
    /// `penFlags` as Windows gives them.
    pub pen_flags: u32,
    /// `pointerInfo.pointerFlags` as Windows gives them.
    pub pointer_flags: u32,
    /// Pressure, 0–1024.
    pub pressure: u32,
}

impl PenState {
    /// The eraser end or eraser button is in use.
    pub fn eraser(&self) -> bool {
        self.pen_flags & (PEN_FLAG_INVERTED | PEN_FLAG_ERASER) != 0
    }

    /// The barrel button is held.
    pub fn barrel(&self) -> bool {
        self.pen_flags & PEN_FLAG_BARREL != 0
    }
}

/// The pen state of Windows pointer `pointer_id`, or `None` when it is not a pen (a finger, a
/// mouse, a pointer that has gone) or off Windows.
#[cfg(windows)]
pub fn pen_state(pointer_id: u32) -> Option<PenState> {
    use windows_sys::Win32::UI::Input::Pointer::{GetPointerPenInfo, POINTER_PEN_INFO};
    // SAFETY: POINTER_PEN_INFO is plain data, all zeros is a valid value, and the call writes at
    // most one struct into the pointer it is given, which lives for the whole call.
    #[allow(unsafe_code)]
    let (ok, info) = unsafe {
        let mut info: POINTER_PEN_INFO = core::mem::zeroed();
        let ok = GetPointerPenInfo(pointer_id, &mut info);
        (ok, info)
    };
    (ok != 0).then_some(PenState { pen_flags: info.penFlags, pointer_flags: info.pointerInfo.pointerFlags, pressure: info.pressure })
}

/// Off Windows there is no pen state to read.
#[cfg(not(windows))]
pub fn pen_state(_pointer_id: u32) -> Option<PenState> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eraser_is_the_inverted_or_eraser_flag() {
        assert!(PenState { pen_flags: PEN_FLAG_INVERTED, ..Default::default() }.eraser());
        assert!(PenState { pen_flags: PEN_FLAG_ERASER, ..Default::default() }.eraser());
        assert!(!PenState { pen_flags: PEN_FLAG_BARREL, ..Default::default() }.eraser());
        assert!(PenState { pen_flags: PEN_FLAG_BARREL, ..Default::default() }.barrel());
    }
}
