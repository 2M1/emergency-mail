use std::{cmp::max, path::Path, process::Command};

use log::{error, info, trace};

use crate::printing::document::Printable;

pub struct PDFFilePrinter<'a> {
    pub path: &'a Path,
}

impl<'a> PDFFilePrinter<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }

    #[cfg(debug_assertions)]
    fn _run_print_cmd(&self, mut binding: Command) {
        // do nothing in debug mode
        trace!("skipping print command in debug mode");
    }

    #[cfg(not(debug_assertions))]
    fn _run_print_cmd(&self, mut binding: Command) {
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
        let mut binding = Command::new(&config.printing.sumatra_path);
        if let Some(printer) = &config.printing.printer {
            binding.arg("-print-to").arg(printer);
        } else {
            binding.arg("-print-to-default");
        };
        binding.arg("-print-settings").arg(format!(
            "{}x",
            max(times, config.printing.min_copies as usize)
        ));

        let command = binding.arg(self.path.to_str().expect("couldn't convert path to string"));
        trace!("command: {:?}", command);
        self._run_print_cmd(binding);
    }
}
