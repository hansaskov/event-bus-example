use std::time::SystemTime;

use crate::event_bus::Reading;
use crate::module::{Module, ModuleCtx};
use anyhow::{Context, Result};
use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum SensorType {
    Voltage,
    Clock,
    Temperature,
    Load,
    Fan,
    Flow,
    Control,
    Level,
}

struct ReadingDefinition {
    name: String,
    sensor_type: SensorType,
    query_name: String,
    exact_match: bool,
    unit: String,
    category: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct Sensor {
    value: f32,
}

pub struct Monitoring {
    ctx: ModuleCtx,
    wmi_con: WMIConnection,
    config: Vec<ReadingDefinition>,
}

impl Module for Monitoring {
    fn new(ctx: ModuleCtx) -> Self {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con =
            WMIConnection::with_namespace_path("ROOT\\LibreHardwareMonitor", com_con).unwrap();

        let config = vec![
            ReadingDefinition {
                name: "CPU Temperature".into(),
                sensor_type: SensorType::Temperature,
                query_name: "Core".into(),
                exact_match: false,
                unit: "C".into(),
                category: "computer".into(),
            },
            ReadingDefinition {
                name: "CPU Usage".into(),
                sensor_type: SensorType::Load,
                query_name: "CPU Total".into(),
                exact_match: true,
                unit: "%".into(),
                category: "computer".into(),
            },
            ReadingDefinition {
                name: "Memory Usage".into(),
                sensor_type: SensorType::Load,
                query_name: "Memory".into(),
                exact_match: true,
                unit: "%".into(),
                category: "computer".into(),
            },
        ];

        Monitoring {
            ctx,
            wmi_con,
            config,
        }
    }

    async fn run(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match self.get_all_readings() {
                        Ok(readings) => readings.iter().for_each(|reading| self.ctx.send_reading(reading.clone())),
                        Err(e) => self.ctx.send_message(e.to_string()),
                    }
                },
            }
        }
    }
}

impl Monitoring {
    fn get_query(sensor_type: &SensorType, name_filter: &str, exact_match: bool) -> String {
        let comparison = if exact_match { "=" } else { "LIKE" };
        let value = if exact_match {
            name_filter.to_string()
        } else {
            format!("%{}%", name_filter)
        };
        format!(
            "SELECT * FROM Sensor WHERE SensorType = '{:?}' AND Name {comparison} '{value}'",
            sensor_type
        )
    }

    fn get_sensor(&self, query: &str) -> Result<Sensor> {
        let result: Vec<Sensor> = self.wmi_con.raw_query(query)?;
        result
            .first()
            .cloned()
            .context("Sensor not found. Is Libre Hardware Monitor running?")
    }

    fn get_reading(&self, definition: &ReadingDefinition) -> Result<Reading> {
        let query = Self::get_query(
            &definition.sensor_type,
            &definition.query_name,
            definition.exact_match,
        );
        let sensor = self.get_sensor(&query)?;
        let timestamp = SystemTime::now();

        Ok(Reading {
            time: timestamp,
            name: definition.name.to_string(),
            value: sensor.value,
            unit: definition.unit.to_string(),
            category: definition.category.to_string(),
        })
    }

    fn get_all_readings(&self) -> Result<Vec<Reading>> {
        self.config
            .iter()
            .map(|def| self.get_reading(def))
            .collect()
    }
}
