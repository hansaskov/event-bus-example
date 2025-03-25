use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::time;
use wmi::{COMLibrary, WMIConnection};

use crate::module::{Module, ModuleCtx};
use crate::reading::Reading;

/// Sensor types for hardware monitoring
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum SensorType {
    Temperature,
    Load,
}

/// Configuration for a hardware sensor
struct SensorConfig {
    category: &'static str,
    name: &'static str,
    unit: &'static str,
    sensor_type: SensorType,
    query_name: &'static str,
}

/// Hardware monitoring module
pub struct Monitoring {
    ctx: ModuleCtx,
    wmi_con: WMIConnection,
    sensors: Vec<SensorConfig>,
}

/// Represents raw sensor data from WMI
#[derive(Deserialize)]
struct WmiSensor {
    value: f32,
}

impl Monitoring {
    pub fn new(ctx: ModuleCtx) -> Self {
        let com_con = COMLibrary::new()
            .context("Failed to initialize COM library")
            .expect("COM library initialization");

        let wmi_con = WMIConnection::with_namespace_path("ROOT\\LibreHardwareMonitor", com_con)
            .context("Failed to connect to WMI namespace")
            .expect("WMI connection");

        Self {
            ctx,
            wmi_con,
            sensors: Self::sensor_configs(),
        }
    }

    /// Default sensor configurations
    fn sensor_configs() -> Vec<SensorConfig> {
        vec![
            SensorConfig {
                category: "computer",
                name: "CPU Temperature",
                unit: "°C",
                sensor_type: SensorType::Temperature,
                query_name: "Core",
            },
            SensorConfig {
                category: "computer",
                name: "CPU Usage",
                unit: "%",
                sensor_type: SensorType::Load,
                query_name: "CPU Total",
            },
            SensorConfig {
                category: "computer",
                name: "Memory Usage",
                unit: "%",
                sensor_type: SensorType::Load,
                query_name: "Memory",
            },
        ]
    }

    /// Builds a WMI query for a specific sensor
    fn build_query(config: &SensorConfig) -> String {
        format!(
            "SELECT * FROM Sensor WHERE SensorType = '{:?}' AND Name LIKE '%{}%'",
            config.sensor_type, config.query_name
        )
    }

    /// Fetches a reading for a single sensor
    fn fetch_reading(&self, config: &SensorConfig) -> Result<Reading> {
        let query = Self::build_query(config);
        let sensors: Vec<WmiSensor> = self
            .wmi_con
            .raw_query(&query)
            .context("Failed to query sensor data")?;

        // Select the first value.
        let value = sensors
            .first()
            .context(format!(
                "No data found for {}. \t Is LibreHardwareMonitor Running?",
                config.query_name
            ))?
            .value;

        Ok(Reading {
            time: std::time::SystemTime::now(),
            category: config.category.to_string(),
            name: config.name.to_string(),
            unit: config.unit.to_string(),
            value,
        })
    }
}

impl Module for Monitoring {
    /// Runs the monitoring loop
    async fn run(&mut self) -> Result<()> {
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    for sensor in &self.sensors {
                        match self.fetch_reading(sensor) {
                            Ok(reading) => self.ctx.send_reading(reading),
                            Err(e) => self.ctx.send_log(format!("{e}")),
                        }
                    }

                }
            }
        }
    }
}
