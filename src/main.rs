use std::error::Error;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::{fmt, fs, thread};

/// Custom error type for GPU information retrieval errors.
#[derive(Debug)]
struct GpuInfoError(String);

impl Display for GpuInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for GpuInfoError {}

/// Struct to hold GPU data.
struct GpuData {
    temperature: String,
    core_clock: String,
    power_usage: String,
    gpu_load: String,
    vram_used: String,
    vram_total: String,
}

/// Detects AMD GPUs by checking the vendor ID in the DRM directory.
fn detect_amd_gpus() -> Result<Vec<PathBuf>, Box<dyn Error + Send + Sync>> {
    let mut amd_gpus = Vec::new();
    let drm_dir = Path::new("/sys/class/drm");
    for entry in fs::read_dir(drm_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("card") {
                let vendor_path = path.join("device/vendor");
                if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                    if vendor.trim() == "0x1002" {
                        amd_gpus.push(path);
                    }
                }
            }
        }
    }
    Ok(amd_gpus)
}

/// Finds the hwmon directory for a given GPU path.
fn find_hwmon_dir(gpu_path: &Path) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let hwmon_base = gpu_path.join("device/hwmon");
    for entry in fs::read_dir(hwmon_base)? {
        let entry = entry?;
        let hwmon_path = entry.path();
        if hwmon_path.is_dir() {
            let temp_path = hwmon_path.join("temp1_input");
            if temp_path.exists() {
                return Ok(hwmon_path);
            }
        }
    }

    Err(Box::new(GpuInfoError("No hwmon directory found".into())))
}

/// Reads and parses a value from a file at the given path.
fn read_and_parse<T>(path: &Path) -> Result<T, Box<dyn Error + Send + Sync>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static + Send + Sync,
{
    let s = fs::read_to_string(path)?;
    let value = s.trim().parse::<T>()?;

    Ok(value)
}

/// Formats a frequency value in Hz to a human-readable string with appropriate units.
fn format_frequency(hz: u64) -> String {
    let units = ["Hz", "KHz", "MHz", "GHz"];
    let mut value = hz as f64;
    let mut unit_index = 0;

    while value >= 1000.0 && unit_index < units.len() - 1 {
        value /= 1000.0;
        unit_index += 1;
    }

    let rounded_value = value.round();
    format!("{} {}", rounded_value as u64, units[unit_index])
}

/// Reads GPU data such as temperature, core clock, power usage, etc.
fn read_gpu_data(
    gpu_path: &Path,
    hwmon_dir: &Path,
) -> Result<GpuData, Box<dyn Error + Send + Sync>> {
    // Read temperature in millidegrees Celsius and convert to degrees Celsius.
    let temp_path = hwmon_dir.join("temp1_input");
    let temp_milli: i32 = read_and_parse(&temp_path)?;
    let temperature = format!("{}°C", temp_milli / 1000);

    // Read core clock frequency in Hz and format it.
    let freq_path = hwmon_dir.join("freq1_input");
    let freq_hz: u64 = read_and_parse(&freq_path)?;
    let core_clock = format_frequency(freq_hz);

    // Read power usage in microwatts and convert to watts.
    let power_path = hwmon_dir.join("power1_average");
    let power_microwatts: u64 = read_and_parse(&power_path)?;
    let power_usage = format!("{} Watts", power_microwatts / 1_000_000);

    // Read GPU load percentage.
    let load_path = gpu_path.join("device/gpu_busy_percent");
    let load_percent: f32 = read_and_parse(&load_path)?;
    let gpu_load = format!("{:.1}%", load_percent);

    // Read VRAM usage and total VRAM in bytes, convert to MB.
    let vram_used_path = gpu_path.join("device/mem_info_vram_used");
    let vram_total_path = gpu_path.join("device/mem_info_vram_total");
    let vram_used: u64 = read_and_parse(&vram_used_path)?;
    let vram_total: u64 = read_and_parse(&vram_total_path)?;

    let vram_used_mb = format!("{} MB", vram_used / 1024);
    let vram_total_mb = format!("{} MB", vram_total / 1024);

    Ok(GpuData {
        temperature,
        core_clock,
        power_usage,
        gpu_load,
        vram_used: vram_used_mb,
        vram_total: vram_total_mb,
    })
}

/// Main function to run the GPU data retrieval and display the results.
fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Detect AMD GPUs.
    let amd_gpus = detect_amd_gpus()?;
    if amd_gpus.is_empty() {
        println!("No AMD GPUs detected.");
        return Ok(());
    }

    // Spawn a thread for each GPU to read its data concurrently.
    let handles: Vec<_> = amd_gpus
        .into_iter()
        .map(|gpu_path| {
            thread::spawn(move || {
                let hwmon_dir = find_hwmon_dir(&gpu_path)?;
                read_gpu_data(&gpu_path, &hwmon_dir)
            })
        })
        .collect();

    // Collect and print the results from each thread.
    for handle in handles {
        match handle.join().unwrap() {
            Ok(gpu_data) => {
                let json_output = format!(
                    r#"{{"GPU Temperature": "{}", "GPU Load": "{}", "GPU Core Clock": "{}", "GPU Power Usage": "{}", "GPU VRAM Usage": "{}", "GPU VRAM Total": "{}"}}"#,
                    gpu_data.temperature,
                    gpu_data.gpu_load,
                    gpu_data.core_clock,
                    gpu_data.power_usage,
                    gpu_data.vram_used,
                    gpu_data.vram_total
                );
                println!("{}", json_output);
            }
            Err(e) => eprintln!("Error reading GPU data: {}", e),
        }
    }

    Ok(())
}

/// Entry point of the application.
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
