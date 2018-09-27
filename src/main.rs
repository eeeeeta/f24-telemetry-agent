extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate unbounded_gpsd;
#[macro_use] extern crate failure;
extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate hyper_tls;
#[macro_use] extern crate log;
extern crate env_logger;

mod proto;
mod gpsd;
mod arduino;

use futures::{Future, Async, Poll, Stream};
use futures::sync::mpsc::{UnboundedReceiver, self};
use self::proto::UploadBody;
use hyper::Client;
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;
use tokio_core::reactor::{Handle, Core};
use hyper::{Request};
use std::thread;
use std::time::Duration;
use self::arduino::ArduinoHandler;
use self::gpsd::GpsdHandler;

#[derive(Deserialize)]
pub struct Config {
    base_url: String,
    car_identifier: String,
    access_token: String,
    arduino_path: String,
    gpsd_addr: String,
}
pub struct TelemetryAgent {
    rx: UnboundedReceiver<UploadBody>,
    cli: Client<HttpsConnector<HttpConnector>>,
    cfg: Config,
    req: usize,
    hdl: Handle
}
impl Future for TelemetryAgent {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<(), failure::Error> {
        while let Async::Ready(val) = self.rx.poll().unwrap() {
            let val = val.expect("Data receiver died");
            self.upload(val)?;
        }
        Ok(Async::NotReady)
    }
}
impl TelemetryAgent {
    pub fn upload(&mut self, body: UploadBody) -> Result<(), failure::Error> {
        let rn = self.req;
        self.req += 1;
        info!("[POST-{}] Uploading data, ts: {}", rn, body.ts);
        let body = serde_json::to_vec(&body)?;
        let req = Request::builder()
            .method("POST")
            .uri(format!("{}/_telemetry/r0/data/{}/upload?access_token={}", self.cfg.base_url, self.cfg.car_identifier, self.cfg.access_token))
            .header("Content-Type", "application/json")
            .body(body.into())?;
        let fut = self.cli.request(req)
            .then(move |res| {
                match res {
                    Ok(r) => {
                        let st = r.status();
                        if !st.is_success() {
                            warn!("[POST-{}] Request failed with status {}", rn, st)
                        }
                        else {
                            info!("[POST-{}] Uploaded, status {}", rn, st);
                        }
                    },
                    Err(e) => {
                        warn!("[POST-{}] Request failed: {}", rn, e);
                    }
                }
                Ok(())
            });
        self.hdl.spawn(fut);
        Ok(())
    }
}
fn main() {
    env_logger::init();
    info!("[+] F24 Telemetry Agent");
    info!("[+] Reading configuration");
    let config = std::env::args().nth(1).expect("Config file should be first argument");
    let cfg_text = std::fs::read_to_string(&config).unwrap();
    let cfg: Config = toml::from_str(&cfg_text).unwrap();
    info!("[+] Setting up data sources");
    let (arduino_tx, rx) = mpsc::unbounded();
    let gpsd_tx = arduino_tx.clone();
    let path = cfg.arduino_path.clone();
    let addr = cfg.gpsd_addr.clone();
    thread::spawn(move || {
        loop {
            info!("[+] Setting up Arduino");
            match ArduinoHandler::new(&path, arduino_tx.clone()) {
                Ok(a) => {
                    if let Err(e) = a.run() {
                        error!("[A] Arduino failed: {}", e);
                        thread::sleep(Duration::from_millis(1000));
                    }
                },
                Err(e) => {
                    error!("[A] Arduino failed setup: {}", e);
                    thread::sleep(Duration::from_millis(5000));
                }
            }
        }
    });
    thread::spawn(move || {
        loop {
            info!("[+] Setting up gpsd");
            match GpsdHandler::new(&addr, gpsd_tx.clone()) {
                Ok(a) => {
                    if let Err(e) = a.run() {
                        error!("[G] gpsd failed: {}", e);
                        thread::sleep(Duration::from_millis(1000));
                    }
                },
                Err(e) => {
                    error!("[G] gpsd failed setup: {}", e);
                    thread::sleep(Duration::from_millis(5000));
                }
            }
        }
    });
    info!("[+] Setting up uploader");
    let mut core = Core::new().unwrap();
    let hdl = core.handle();
    let https = HttpsConnector::new(4).unwrap();
    let client = Client::builder()
        .build::<_, hyper::Body>(https);
    let agent = TelemetryAgent {
        rx,
        cli: client,
        cfg,
        req: 0,
        hdl
    };
    info!("[+] Running telemetry agent");
    core.run(agent).unwrap();
}
