/// Simple logger that respects verbosity levels
pub struct Logger {
    verbosity: u8,
}

impl Logger {
    pub fn new(verbosity: u8) -> Self {
        Self { verbosity }
    }

    pub fn info(&self, message: &str) {
        if self.verbosity >= 1 {
            println!("{}", message);
        }
    }

    pub fn debug(&self, message: &str) {
        if self.verbosity >= 2 {
            eprintln!("DEBUG: {}", message);
        }
    }

    pub fn trace(&self, message: &str) {
        if self.verbosity >= 3 {
            eprintln!("TRACE: {}", message);
        }
    }

    pub fn success(&self, message: &str) {
        println!("✅ {}", message);
    }

    #[allow(dead_code)]
    pub fn warning(&self, message: &str) {
        eprintln!("⚠️  {}", message);
    }

    #[allow(dead_code)]
    pub fn error(&self, message: &str) {
        eprintln!("❌ {}", message);
    }
}
