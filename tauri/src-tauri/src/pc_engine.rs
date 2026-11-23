//! Motor de secuencia eléctrica en la PC sobre el PLC puente DB100.
//!
//! El PLC solo copia E/S. Toda la secuencia, rangos, tiempos y evaluación
//! se ejecutan aquí. Las analógicas se convierten provisionalmente con
//! fondo de escala; la pantalla de calibración las ajustará después.

use crate::plc::{self, PlcClient};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const ADC_FULL_SCALE: f64 = 27648.0;
const VOLTAGE_TOLERANCE: f64 = 0.05;

#[derive(Clone, Debug)]
pub struct SequenceOptions {
    pub voltage: f64,
    pub test_time: f64,
    pub estop_nc: bool,
}

#[derive(Clone, Debug)]
pub struct SequenceResult {
    pub pass: bool,
    pub values: Value,
    pub readings: Value,
    pub errors: Vec<String>,
}

struct ComponentResult {
    name: String,
    value: f64,
    raw: u16,
    min: f64,
    max: f64,
    pass: bool,
    range: &'static str,
}

fn component_output(name: &str) -> Option<&'static str> {
    match name {
        "FAN" => Some("Q0.5"),
        "DOOR" => Some("Q1.0"),
        "FRAME" => Some("Q0.7"),
        "DRAIN" => Some("Q1.1"),
        "DEFROST" => Some("Q2.0"),
        "LIGHT" => Some("Q0.6"),
        _ => None,
    }
}

fn range_for(max_current: f64) -> (&'static str, &'static str, f64) {
    if max_current <= 1.0 {
        ("A", "Q0.3", 1.0)
    } else if max_current <= 10.0 {
        ("B", "Q0.4", 10.0)
    } else {
        ("C", "Q0.2", 20.0)
    }
}

fn raw_to_units(raw: u16, full_scale: f64) -> f64 {
    (raw as f64 / ADC_FULL_SCALE) * full_scale
}

fn output_off_all(client: &mut PlcClient) {
    for q in ["Q0.0", "Q0.1", "Q0.2", "Q0.3", "Q0.4", "Q0.5", "Q0.6", "Q0.7",
              "Q1.0", "Q1.1", "Q2.0"] {
        let _ = plc::write_bridge_output(client, q, false);
    }
}

fn read_bridge(client: &mut PlcClient) -> Result<Value, String> {
    plc::read_bridge(client)
}

fn estop_active(client: &mut PlcClient, nc: bool) -> Result<bool, String> {
    let v = read_bridge(client)?;
    let raw = v.get("estop").and_then(|x| x.as_bool()).unwrap_or(false);
    Ok(if nc { !raw } else { raw })
}

fn sleep_with_estop(client: &mut PlcClient, total: Duration, nc: bool) -> Result<(), String> {
    let start = Instant::now();
    while start.elapsed() < total {
        if estop_active(client, nc)? {
            return Err("Paro de emergencia activo".to_string());
        }
        let remaining = total.saturating_sub(start.elapsed());
        std::thread::sleep(remaining.min(Duration::from_millis(100)));
    }
    Ok(())
}

fn read_analog(client: &mut PlcClient, channel: &str) -> Result<u16, String> {
    let v = read_bridge(client)?;
    v.get("raw_analog")
        .and_then(|a| a.as_array())
        .and_then(|a| match channel {
            "IA1" => a.first(),
            "IA2" => a.get(1),
            "IA3" => a.get(2),
            "IA4" => a.get(3),
            _ => None,
        })
        .and_then(|x| x.as_u64())
        .map(|x| x as u16)
        .ok_or_else(|| format!("No se pudo leer {}", channel))
}

/// Ejecuta la secuencia eléctrica. El Hipot ya debe haber pasado.
pub fn run_electrical(
    app: &AppHandle,
    client: &mut PlcClient,
    components: &Value,
    opts: &SequenceOptions,
) -> Result<SequenceResult, String> {
    let mut errors: Vec<String> = Vec::new();
    let mut component_results: Vec<ComponentResult> = Vec::new();

    output_off_all(client);
    if estop_active(client, opts.estop_nc)? {
        output_off_all(client);
        return Err("Paro de emergencia activo antes de iniciar la secuencia".to_string());
    }

    // Selección de voltaje. El PLC antiguo usaba KM2=120 y KM1=220.
    let use_120 = opts.voltage < 150.0;
    plc::write_bridge_output(client, "Q0.0", use_120)?;
    plc::write_bridge_output(client, "Q0.1", !use_120)?;
    let expected_voltage = if use_120 { 120.0 } else { 220.0 };

    // Estabilización de voltaje: 2.4 s, como el PLC anterior.
    sleep_with_estop(client, Duration::from_millis(2400), opts.estop_nc)?;
    let v_raw = read_analog(client, "IA4")?;
    let measured_voltage = raw_to_units(v_raw, 220.0);
    let voltage_ok = measured_voltage >= expected_voltage * (1.0 - VOLTAGE_TOLERANCE)
        && measured_voltage <= expected_voltage * (1.0 + VOLTAGE_TOLERANCE);

    for comp in components.as_array().cloned().unwrap_or_default() {
        let name = comp
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_uppercase();
        if name.is_empty() || !comp.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true) {
            continue;
        }
        let Some(output) = component_output(&name) else {
            errors.push(format!("Componente sin salida configurada: {}", name));
            continue;
        };
        let min = comp.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let max = comp.get("max").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let (range, range_output, full_scale) = range_for(max);
        let channel = match range {
            "A" => "IA1",
            "B" => "IA2",
            _ => "IA3",
        };

        if estop_active(client, opts.estop_nc)? {
            output_off_all(client);
            return Err("Paro de emergencia activo durante la secuencia".to_string());
        }

        plc::write_bridge_output(client, output, true)?;
        plc::write_bridge_output(client, range_output, true)?;

        let test_time = if opts.test_time > 0.0 { opts.test_time } else { 10.0 };
        sleep_with_estop(client, Duration::from_secs_f64(test_time), opts.estop_nc)?;

        let raw = read_analog(client, channel)?;
        let value = raw_to_units(raw, full_scale);
        let pass = value >= min && value <= max;
        let mut snapshot = serde_json::Map::new();
        snapshot.insert(name.clone(), json!(value));
        let _ = app.emit("test-values", Value::Object(snapshot));

        plc::write_bridge_output(client, range_output, false)?;
        plc::write_bridge_output(client, output, false)?;
        component_results.push(ComponentResult {
            name,
            value,
            raw,
            min,
            max,
            pass,
            range,
        });
    }

    plc::write_bridge_output(client, "Q0.0", false)?;
    plc::write_bridge_output(client, "Q0.1", false)?;

    let mut values = serde_json::Map::new();
    let mut readings = Vec::new();
    for r in &component_results {
        values.insert(r.name.clone(), json!(r.value));
        readings.push(json!({
            "component": r.name,
            "tag": r.name,
            "value": r.value,
            "raw": r.raw,
            "min": r.min,
            "max": r.max,
            "range": r.range,
            "pass": r.pass,
        }));
    }
    let all_ok = !component_results.is_empty() && component_results.iter().all(|r| r.pass);
    let pass = voltage_ok && all_ok;
    if !voltage_ok {
        errors.push(format!(
            "Voltaje fuera de rango: {:.1} V (esperado {:.0} V)",
            measured_voltage, expected_voltage
        ));
    }
    Ok(SequenceResult {
        pass,
        values: Value::Object(values),
        readings: json!({
            "voltage": measured_voltage,
            "expected_voltage": expected_voltage,
            "voltage_ok": voltage_ok,
            "components": readings,
        }),
        errors,
    })
}
