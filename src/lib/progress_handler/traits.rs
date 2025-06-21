use indicatif::ProgressBar;

pub trait ProgressHandler: Send + Sync {
    fn start(&self, total: usize);
    fn set_size(&self, size: usize);
    fn advance(&self);
    fn finish(&self);
    fn set_message(&self, str: String);
    fn println(&self, str: String);
}
pub struct NullProgressHandler;
impl ProgressHandler for NullProgressHandler {
    fn start(&self, _total: usize) {}
    fn advance(&self) {}
    fn finish(&self) {}
    fn set_message(&self, _str: String) {}
    fn println(&self, _str: String) {}
    fn set_size(&self, _size: usize) {}
}

pub struct CliProgress {
    pub pb: ProgressBar,
}

impl ProgressHandler for CliProgress {
    fn start(&self, total: usize) {
        self.pb.reset();
        self.pb.set_length(total.try_into().unwrap());
    }

    fn set_size(&self, size: usize) {
        self.pb.set_length(size.try_into().unwrap());
    }

    fn advance(&self) {
        self.pb.inc(1);
    }

    fn finish(&self) {
        self.pb.finish_and_clear();
    }

    fn println(&self, str: String) {
        self.pb.println(str);
    }

    fn set_message(&self, str: String) {
        self.pb.set_message(str);
    }
}

impl Default for NullProgressHandler {
    fn default() -> Self {
        Self
    }
}

impl Default for CliProgress {
    fn default() -> Self {
        CliProgress {
            pb: ProgressBar::no_length(),
        }
    }
}
