//! Talking to the Arduino.

use futures::sync::mpsc::UnboundedSender;
use super::proto::UploadBody;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use failure::Error;
use std::time::{SystemTime, UNIX_EPOCH};

fn get_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
pub struct ArduinoHandler {
    tx: UnboundedSender<UploadBody>,
    arduino: BufReader<File>
}
impl ArduinoHandler {
    pub fn new(path: &str, tx: UnboundedSender<UploadBody>) -> Result<Self, Error> {
        let file = File::open(path)?;
        Ok(Self {
            arduino: BufReader::new(file),
            tx
        })
    }
    pub fn run(self) -> Result<(), Error> {
        for line in self.arduino.lines() {
            let line = line?;
            // Data format:
            // arduino_secs, voltage, current, watthours,
            // temp_1, temp_2, rpm1, rpm2
            let mut fragments = line.split(",")
                .map(|x| {
                    if let Ok(d) = x.parse::<f32>() {
                        Some(d)
                    }
                    else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            fragments.resize(8, None);
            let body = UploadBody {
                ts: get_ts(),
                arduino_secs: fragments[0].map(|x| x as i32),
                voltage: fragments[1],
                current: fragments[2],
                watthours: fragments[3],
                temp1: fragments[4],
                temp2: fragments[5],
                rpm1: fragments[6],
                rpm2: fragments[7],
                ..Default::default()
            };
            info!("[A] Got data from Arduino, seconds: {}", body.arduino_secs.unwrap_or(-1));
            self.tx.unbounded_send(body)?;
        }
        Err(format_err!("Serial connection returned EOF"))
    }
}
