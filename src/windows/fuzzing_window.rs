#![cfg(feature = "for_fuzzer")]
use libc;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::{
    any::Any,
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{
    app::Request,
    popup::{Popup, PopupType},
    window::Window,
};

use flashfuzzemu::opts::EmuOpts;
use flashfuzzer::fuzz;

use crossterm::event::KeyEvent;

pub struct FuzzingWindow {
    pub context: Arc<RwLock<FuzzingWindowCtx>>,
    pub properties: HashMap<String, Box<dyn Any>>,
    pub started: bool,
    pub str: String,
}

impl FuzzingWindow {
    pub fn new() -> Self {
        Self {
            context: Arc::new(RwLock::new(FuzzingWindowCtx {})),
            properties: HashMap::new(),
            started: false,
            str: "Nothing".into(),
        }
    }
    fn try_start(&mut self) -> Option<Vec<Request>> {
        self.str.push_str("Trying to start...\n".into());
        let opts = match self.properties.get("emu_opts") {
            None => {
                return Some(vec![Request::GetProperty("emu_opts".into())]);
            }
            Some(o) => match (*o).downcast_ref::<EmuOpts>() {
                None => {
                    return Some(vec![Request::Popup(Popup::new(
                        PopupType::Warning,
                        "Unexpected Type for emu_opts",
                    ))]);
                }
                Some(o) => o.clone(),
            },
        };
        self.str.push_str("Getting port...\n".into());
        let port = match self.properties.get("port") {
            None => {
                return Some(vec![Request::GetProperty("port".into())]);
            }
            Some(p) => match (*p).downcast_ref::<u16>() {
                None => {
                    return Some(vec![Request::Popup(Popup::new(
                        PopupType::Warning,
                        "Unexpected Type for port",
                    ))]);
                }
                Some(p) => *p,
            },
        };
        self.str.push_str("Getting binary...\n".into());
        let binary = match self.properties.get("binary_path") {
            None => {
                return Some(vec![Request::GetProperty("binary_path".into())]);
            }
            Some(b) => match (*b).downcast_ref::<PathBuf>() {
                None => {
                    return Some(vec![Request::Popup(Popup::new(
                        PopupType::Warning,
                        "Unexpected Type for binary_path",
                    ))]);
                }
                Some(b) => b.clone(),
            },
        };
        self.str.push_str("Done!\n".into());
        let mut contents = match std::fs::read(&binary) {
            Ok(c) => c,
            Err(e) => {
                return Some(vec![Request::Popup(Popup::new(
                    PopupType::Warning,
                    format!("Error reading binary file: {}", e),
                ))]);
            }
        };
        self.str = format!(
            "Try start: opts: {:?}, port: {}, binary: {:?}",
            opts, port, binary
        );
        unsafe {
            // TODO: Why fork why not thread
            let pid = libc::fork();
            if pid < 0 {
                return Some(vec![Request::Popup(Popup::new(
                    PopupType::Warning,
                    "Fork failed".to_string(),
                ))]);
            }
            if pid == 0 {
                let _ = fuzz(&mut contents, opts, port as u64);
            }
            self.started = true;
            return None;
        }
    }
}

impl Window for FuzzingWindow {
    fn name(&self) -> &str {
        "Fuzzing"
    }
    fn render(
        &mut self,
        f: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> Option<Vec<Request>> {
        // Render logic for the fuzzing window
        let para = Paragraph::new(self.str.clone()).block(
            Block::default()
                .title("Fuzzing Window")
                .borders(Borders::ALL),
        );
        f.render_widget(para, area);
        if !self.started {
            return self.try_start();
        }
        None
    }
    fn handle_input(&mut self, _key: KeyEvent) -> Option<Vec<crate::app::Request>> {
        // Handle input for the fuzzing window
        None
    }
    fn send_property(&mut self, name: String, property: &dyn Any) {
        self.str.push_str(&format!(
            "Property received: {} - {:?}\n",
            name,
            std::any::type_name_of_val(property)
        ));
        let boxed: Option<Box<dyn Any>> = if let Some(p) = property.downcast_ref::<PathBuf>() {
            Some(Box::new(p.clone()))
        } else if let Some(p) = property.downcast_ref::<u16>() {
            self.str
                .push_str(&format!("Property {} set successfully as u16.\n", &name));
            Some(Box::new(*p))
        } else if let Some(p) = property.downcast_ref::<u64>() {
            self.str
                .push_str(&format!("Property {} set successfully as u64.\n", &name));
            Some(Box::new(*p))
        } else if let Some(p) = property.downcast_ref::<String>() {
            self.str
                .push_str(&format!("Property {} set successfully as String.\n", &name));

            Some(Box::new(p.clone()))
        } else if let Some(p) = property.downcast_ref::<EmuOpts>() {
            Some(Box::new(p.clone()))
        } else {
            self.str
                .push_str(&format!("Unsupported property type for: {}\n", name));
            None
        };

        if let Some(val) = boxed {
            self.properties.insert(name.clone(), val);
            self.str
                .push_str(&format!("Property {} set successfully.\n", &name));
        }
    }
}

pub struct FuzzingWindowCtx {}
