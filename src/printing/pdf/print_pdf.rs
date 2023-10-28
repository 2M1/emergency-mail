use std::{path::Path, process::Command};

#[cfg(not(debug_assertions))]
use log::error;
use log::{info, trace}; // avoid the unused import warning

use crate::printing::document::Printable;

pub struct PDFFilePrinter<'a> {
    pub path: &'a Path,
}

impl<'a> PDFFilePrinter<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }

    #[cfg(debug_assertions)]
    fn _run_print_cmd(&self, mut _binding: Command) {
        // do nothing in debug mode
        trace!("skipping print command in debug mode");
    }

    #[cfg(not(debug_assertions))]
    fn _run_print_cmd(&self, mut command: Command) {
        let res = command.output();
        if let Err(e) = res {
            error!("couldn't print pdf file: {}", e);
        } else if let Ok(output) = res {
            trace!("printing pdf returned: {}", output.status);
            if !output.status.success() {
                error!(
                    "couldn't print pdf file: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}

impl<'a> Printable for PDFFilePrinter<'a> {
    fn print(&self, times: usize, config: &crate::config::Config) {
        // assumes, that times was computed beforehand and is inside the configured bounds
        debug_assert!(times > 0, "times must be greater than 0");
        debug_assert!(
            times <= config.printing.max_copies.unwrap_or(255) as usize,
            "times must be less than or equal to max_copies"
        );
        debug_assert!(
            times >= config.printing.min_copies as usize,
            "times must be greater than or equal to min_copies"
        );

        let times = if cfg!(debug_assertions) { 1 } else { times };

        if config.printing.disable.unwrap_or(false) {
            info!("printing is disabled");
            return;
        }

        let mut binding = Command::new(&config.printing.sumatra_path);
        if let Some(printer) = &config.printing.printer {
            binding.arg("-print-to").arg(printer);
        } else {
            binding.arg("-print-to-default");
        };
        binding.arg("-print-settings").arg(format!("{}x", times));

        let command = binding.arg(self.path.to_str().expect("couldn't convert path to string"));
        trace!("command: {:?}", command);
        self._run_print_cmd(binding);
    }
}
