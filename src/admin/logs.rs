//! 日志流广播

use std::io::{self, Write};

use tokio::sync::broadcast;

#[derive(Clone)]
pub struct LogBroadcaster {
    sender: broadcast::Sender<String>,
}

impl LogBroadcaster {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    pub fn writer(&self) -> BroadcastWriter {
        BroadcastWriter {
            sender: self.sender.clone(),
        }
    }
}

#[derive(Clone)]
pub struct BroadcastWriter {
    sender: broadcast::Sender<String>,
}

impl Write for BroadcastWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(text) = std::str::from_utf8(buf) {
            let trimmed = text.trim_end();
            if !trimmed.is_empty() {
                let _ = self.sender.send(trimmed.to_string());
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
