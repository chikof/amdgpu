use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{fs, thread};

use errors::GpuInfoError;
use serialize::Json;
use usage::{Units, format_units};

mod errors;
mod serialize;
mod usage;

/// Struct to hold GPU data.
struct GpuData {
    temperature: String,
    core_clock: String,
    power_usage: String,
    gpu_load: String,
    vram_used: String,
    vram_total: String,
}

// Generate JSON serialization and Display via macro
impl_json!(GpuData {
    temperature: "GPU Temperature",
    gpu_load: "GPU Load",
    core_clock: "GPU Core Clock",
    power_usage: "GPU Power Usage",
    vram_used: "GPU VRAM Usage",
    vram_total: "GPU VRAM Total",
});

/// Reads, parses & formats one metric from sysfs.
fn read_metric<T, F>(
    base: &Path,
    rel_path: &str,
    transform: F,
) -> Result<f64, Box<dyn Error + Send + Sync>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static + Send + Sync,
    F: Fn(T) -> f64,
{
    let path = base.join(rel_path);
    let raw: T = fs::read_to_string(&path)?
        .trim()
        .parse()?;
    Ok(transform(raw))
}

/// Reads GPU data such as temperature, clocks, power, load, and VRAM.
fn read_gpu_data(
    gpu_path: &Path,
    hwmon_dir: &Path,
) -> Result<GpuData, Box<dyn Error + Send + Sync>> {
    let temp_val = read_metric::<i32, _>(hwmon_dir, "temp1_input", |m| m as f64 / 1000.0)?;
    let temperature = format_units(Units::Temperature, temp_val);

    let freq_val = read_metric::<f64, _>(hwmon_dir, "freq1_input", |hz| hz)?;
    let core_clock = format_units(Units::Frequency, freq_val);

    let power_val = read_metric::<f64, _>(hwmon_dir, "power1_average", |u| u / 1_000_000.0)?;
    let power_usage = format_units(Units::Power, power_val);

    let load_val =
        read_metric::<f32, _>(gpu_path, "device/gpu_busy_percent", |p| p as f64 / 100.0)?;
    let gpu_load = format_units(Units::Gpu, load_val);

    let used_val = read_metric::<f64, _>(gpu_path, "device/mem_info_vram_used", |b| b)?;
    let vram_used = format_units(Units::Memory, used_val);

    let total_val = read_metric::<f64, _>(gpu_path, "device/mem_info_vram_total", |b| b)?;
    let vram_total = format_units(Units::Memory, total_val);

    Ok(GpuData {
        temperature,
        core_clock,
        power_usage,
        gpu_load,
        vram_used,
        vram_total,
    })
}

/// Detects AMD GPUs by checking vendor IDs under /sys/class/drm.
fn detect_amd_gpus() -> Result<Vec<PathBuf>, Box<dyn Error + Send + Sync>> {
    let gpus = fs::read_dir("/sys/class/drm")?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("card"))
        })
        .filter(|p| {
            fs::read_to_string(p.join("device/vendor"))
                .map(|v| v.trim() == "0x1002")
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    Ok(gpus)
}

/// Finds the hwmon directory containing temperature files for a GPU.
fn find_hwmon_dir(gpu_path: &Path) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let base = gpu_path.join("device/hwmon");
    for entry in fs::read_dir(&base)? {
        let d = entry?.path();
        if d.join("temp1_input")
            .exists()
        {
            return Ok(d);
        }
    }
    Err(Box::new(GpuInfoError("No hwmon directory found".into())))
}

/// Main logic: detect GPUs, read stats concurrently, print JSON.
fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let gpus = detect_amd_gpus()?;
    if gpus.is_empty() {
        println!("No AMD GPUs detected.");
        return Ok(());
    }

    let handles = gpus
        .into_iter()
        .map(|gpu| {
            thread::spawn(move || {
                let hwmon = find_hwmon_dir(&gpu)?;
                read_gpu_data(&gpu, &hwmon)
            })
        });

    for h in handles {
        match h
            .join()
            .unwrap()
        {
            Ok(data) => println!("{}", data.to_json()),
            Err(e) => eprintln!("Error reading GPU data: {}", e),
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Fatal: {}", e);
        exit(1);
    }
}
