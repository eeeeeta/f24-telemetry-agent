//! Communication with GPSD.

use futures::sync::mpsc::UnboundedSender;
use super::proto::UploadBody;
use unbounded_gpsd::GpsdConnection;
use unbounded_gpsd::types::{TpvResponse, Response};
use unbounded_gpsd::errors::GpsdResult;

pub struct GpsdHandler {
    tx: UnboundedSender<UploadBody>,
    conn: GpsdConnection
}

impl GpsdHandler {
    pub fn new(conn: &str, tx: UnboundedSender<UploadBody>) -> GpsdResult<Self> {
        let conn = GpsdConnection::new(conn)?;
        Ok(Self {
            conn, tx
        })
    }
    pub fn run(mut self) -> GpsdResult<()> {
        self.conn.watch(true)?;
        loop {
            let resp = self.conn.get_response()?;
            match resp {
                Response::Tpv(tpv) => {
                    match tpv {
                        TpvResponse::Fix3D { time, lat, lon, speed, .. } |
                        TpvResponse::Fix2D { time, lat, lon, speed, .. } => {
                            let ts = time.timestamp() as u64;
                            info!("[G] Got new GPS TPV response, ts: {}", ts);
                            let body = UploadBody {
                                ts,
                                gps_speed: Some(speed as _),
                                gps_long: Some(lon as _),
                                gps_lat: Some(lat as _),
                                ..Default::default()
                            };
                            self.tx.unbounded_send(body)
                                .map_err(|e| e.to_string())?;
                        },
                        other => {
                            info!("[G] Got non-fix TPV response: {:?}", other);
                        }
                    }
                },
                _ => {}
            }
        }
    }
}
