//! Protocol definitions for the telemetry server.

#[derive(Serialize, Default)]
pub struct UploadBody {
    pub ts: u64,
    #[serde(default)]
    pub arduino_secs: Option<i32>,
    #[serde(default)]
    pub voltage: Option<f32>,
    #[serde(default)]
    pub current: Option<f32>,
    #[serde(default)]
    pub watthours: Option<f32>,
    #[serde(default)]
    pub temp1: Option<f32>,
    #[serde(default)]
    pub temp2: Option<f32>,
    #[serde(default)]
    pub gps_speed: Option<f32>,
    #[serde(default)]
    pub gps_long: Option<f32>,
    #[serde(default)]
    pub gps_lat: Option<f32>,
    #[serde(default)]
    pub accel: Option<f32>,
    #[serde(default)]
    pub pressure1: Option<f32>,
    #[serde(default)]
    pub pressure2: Option<f32>,
    #[serde(default)]
    pub rpm1: Option<f32>,
    #[serde(default)]
    pub rpm2: Option<f32>,
}

