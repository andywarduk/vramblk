use std::error::Error;

use opencl3::{
    command_queue::CommandQueue,
    context::Context,
    device::{get_all_devices, Device, CL_DEVICE_TYPE_GPU},
};

pub fn initialise_cl(gpu: Option<u16>) -> Result<(Context, CommandQueue), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    println!("Probing CL GPU devices");

    let devices = get_all_devices(CL_DEVICE_TYPE_GPU)
        .map_err(|e| format!("Failed to get list of GPU devices: {e}"))?;

    let device_id = match gpu {
        Some(idx) => {
            // Use the specified GPU
            if (idx as usize) < devices.len() {
                devices[idx as usize]
            } else {
                Err(format!("GPU device {idx} does not exist"))?
            }
        }
        None => {
            // Find an appropriate GPU
            match devices.first() {
                Some(dev) => *dev,
                None => Err("No GPU devices found")?,
            }
        }
    };

    // Create GPU device
    let device = Device::new(device_id);

    // Get name
    #[cfg(debug_assertions)]
    let name = device.name().unwrap_or("Unknown".to_string());

    // Create a Context from the GPU device
    #[cfg(debug_assertions)]
    println!("Creating CL context on device {name}");

    let context =
        Context::from_device(&device).map_err(|e| format!("Failed to create CL context: {e}"))?;

    // Create a command_queue on the Context's device
    #[cfg(debug_assertions)]
    println!("Creating CL command queue");

    let queue = CommandQueue::create_default_with_properties(
        &context,
        0,
        device.queue_on_device_preferred_size()? as u32,
    )
    .map_err(|e| format!("Failed to create command queue: {e}"))?;

    Ok((context, queue))
}

pub fn list_devices() -> Result<(), Box<dyn Error>> {
    println!("Available GPU devices:");

    // Loop GPU devices
    for (i, device_id) in get_all_devices(CL_DEVICE_TYPE_GPU)
        .map_err(|e| format!("Failed to get list of GPU devices: {e}"))?
        .into_iter()
        .enumerate()
    {
        let device = Device::new(device_id);

        // Get device name
        let name = device.name().unwrap_or("Unknown".to_string());

        // Get device global memory size
        let mut mem = device.global_mem_size().unwrap_or(0) as f64;

        // Format memory size
        let mut power = 0;

        while mem > 1024_f64 {
            power += 1;
            mem /= 1024_f64;
        }

        let unit = match power {
            0 => "bytes".to_string(),
            1 => "kB".to_string(),
            2 => "MB".to_string(),
            3 => "GB".to_string(),
            4 => "TB".to_string(),
            _ => format!(" x 2^{}", power * 10),
        };

        let precision = if power > 0 { 2 } else { 0 };

        // Print details
        println!("  {i}: {name}, memory {mem:.0$} {unit}", precision)
    }

    Ok(())
}
