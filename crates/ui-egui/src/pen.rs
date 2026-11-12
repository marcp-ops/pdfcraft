//! The pen's eraser end. winit gives a Windows pen to egui as a touch and drops whether it is
//! turned over; [`PenWatch`] asks Windows for that with the touch's pointer id
//! (`pdfcraft_pen::pen_state`), so turning the pen over can switch to the Eraser. It also shows a
//! small diagnostic window (F9) with what the pen reports, to see what a given pen driver sends.

/// What the pen did lately, read from the raw input each frame.
#[derive(Debug)]
pub struct PenWatch {
    /// The eraser end (or eraser button) is in use by the pen touching now.
    pub eraser: bool,
    /// The diagnostic window is showing (F9 toggles it).
    pub show: bool,
    touches: u64,
    last_id: Option<u64>,
    last_state: Option<pdfcraft_pen::PenState>,
    last_phase: Option<egui::TouchPhase>,
}

impl Default for PenWatch {
    fn default() -> Self {
        Self { eraser: false, show: true, touches: 0, last_id: None, last_state: None, last_phase: None }
    }
}

impl PenWatch {
    /// Look at this frame's touches before egui does, so the tool is switched before the stroke
    /// starts.
    pub fn observe(&mut self, raw: &egui::RawInput) {
        for event in &raw.events {
            match event {
                egui::Event::Touch { id, phase, .. } => {
                    self.touches = self.touches.saturating_add(1);
                    self.last_id = Some(id.0);
                    self.last_phase = Some(*phase);
                    let state = u32::try_from(id.0).ok().and_then(pdfcraft_pen::pen_state);
                    self.last_state = state;
                    match phase {
                        egui::TouchPhase::Start | egui::TouchPhase::Move => {
                            if let Some(s) = state {
                                self.eraser = s.eraser();
                            }
                        }
                        // Lifted: the next touch says again which end it is.
                        egui::TouchPhase::End | egui::TouchPhase::Cancel => {
                            if let Some(s) = state {
                                self.eraser = s.eraser();
                            }
                        }
                    }
                }
                egui::Event::Key { key: egui::Key::F9, pressed: true, repeat: false, .. } => self.show = !self.show,
                _ => {}
            }
        }
    }

    /// The diagnostic window, bottom right.
    pub fn show_window(&self, ctx: &egui::Context) {
        if !self.show {
            return;
        }
        let yes = |b: bool| if b { "JA" } else { "nein" };
        egui::Window::new("Stift-Diagnose (F9)")
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-12.0, -12.0))
            .resizable(false)
            .collapsible(true)
            .show(ctx, |ui| {
                ui.label(format!("Touch-Ereignisse: {}", self.touches));
                ui.label(format!("Pointer-ID: {}", self.last_id.map_or_else(|| "–".to_owned(), |i| i.to_string())));
                ui.label(format!("Phase: {:?}", self.last_phase));
                match self.last_state {
                    Some(s) => {
                        ui.label(format!("penFlags: 0x{:X}", s.pen_flags));
                        ui.label(format!("pointerFlags: 0x{:X}", s.pointer_flags));
                        ui.label(format!("Druck: {}", s.pressure));
                        ui.label(format!("Taste: {}", yes(s.barrel())));
                    }
                    None => {
                        ui.label("Kein Stift erkannt (Finger, Maus oder kein Windows).");
                    }
                }
                ui.label(format!("Radierer: {}", yes(self.eraser)));
            });
    }
}
