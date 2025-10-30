use crate::lib::logger::traits::LogLevel;

pub enum WorkerMessage {
    Progress(ProgressMessage),
    Log(LogLevel, String),
}
pub enum ProgressMessage {
    Total(ProgressChangeMessage),
    Current(ProgressChangeMessage),
}

pub enum ProgressChangeMessage {
    SetMessage(String),
    SetSize(usize),
    Start(usize),
    Advance,
    Print(String),
    Finish,
}

impl WorkerMessage {
    pub fn set_total_size(size: usize) -> WorkerMessage {
        WorkerMessage::Progress(ProgressMessage::Total(ProgressChangeMessage::SetSize(size)))
    }

    pub fn set_current_size(size: usize) -> WorkerMessage {
        WorkerMessage::Progress(ProgressMessage::Current(ProgressChangeMessage::SetSize(
            size,
        )))
    }

    pub fn finish_total() -> WorkerMessage {
        WorkerMessage::Progress(ProgressMessage::Total(ProgressChangeMessage::Finish))
    }

    pub fn finish_current() -> WorkerMessage {
        WorkerMessage::Progress(ProgressMessage::Current(ProgressChangeMessage::Finish))
    }

    pub fn log(level: LogLevel, str: String) -> WorkerMessage {
        WorkerMessage::Log(level, str)
    }

    pub fn advance_current() -> WorkerMessage {
        WorkerMessage::Progress(ProgressMessage::Current(ProgressChangeMessage::Advance))
    }

    pub fn advance_total() -> WorkerMessage {
        WorkerMessage::Progress(ProgressMessage::Total(ProgressChangeMessage::Advance))
    }
}
