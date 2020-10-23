use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct Audit {
    pub file_name: String,
    send_channel: Arc<Mutex<Sender<String>>>,
}

impl Audit {
    #[allow(while_true)]
    pub fn new(file_name: &str) -> Self {
        let (tx, rx) = channel::<String>();

        let owned_file_name = file_name.to_owned();

        thread::spawn(move || {
            while true {
                process_message(&rx, &owned_file_name).unwrap_or_else(|err| {
                    error!("Error while writing audit entry: {}", &err.to_string());
                });
            }
        });

        Self {
            file_name: file_name.to_owned(),
            send_channel: Arc::new(Mutex::new(tx)),
        }
    }

    pub fn send_event(&self, txt: String) {
        self.send_channel.lock().unwrap().send(txt).unwrap()
    }
}

fn process_message(
    rx: &Receiver<String>,
    owned_file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = rx.recv()?;

    // save the message
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&owned_file_name)?;

    Ok(writeln!(file, "{}", &message)?)
}
