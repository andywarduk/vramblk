#![deny(missing_docs)]

//! Create NBD GPU ram device

use std::{error::Error, process::ExitCode};

use cl::list_devices;
use clap::Parser;
use libc::{mlockall, sysconf, MCL_CURRENT, MCL_FUTURE, _SC_PAGESIZE};
use parse_size::Config;

mod cl;
mod device;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// List GPU devices
    List,
    /// Mount a VRAM device
    Mount(MountArgs),
}

#[derive(Parser, Debug)]
struct MountArgs {
    /// nbd device to use
    #[clap(short = 'd', long = "device", default_value = "/dev/nbd0", action)]
    device: String,

    /// GPU number to use. Defaults to the first device found
    #[clap(short = 'g', long = "gpu", action)]
    gpu: Option<u16>,

    /// Disk block size. Must be power of 2 between 512 and machine page size. Defaults to machine page size
    #[clap(short = 'b', long = "block-size", default_value_t = page_size(), action, value_parser = parse_block_size_arg)]
    block_size: u64,

    /// Disk size to create - eg. 2097152k, 2048, 2g. Default unit megabytes
    #[clap(action, value_parser = parse_size_arg)]
    size: u64,
}

fn main() -> ExitCode {
    // Prevent this process's memory from being swapped out
    unsafe {
        if mlockall(MCL_CURRENT | MCL_FUTURE) != 0 {
            eprintln!("Warning: Failed to lock process memory");
        }
    }

    // Main process
    match process_command() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::FAILURE;
        }
    }

    return ExitCode::SUCCESS;
}

fn process_command() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = Cli::parse();

    // Execute command
    match args.command {
        Command::List => list_devices()?,
        Command::Mount(mount_args) => {
            // Calculate blocks
            let blocks = ((mount_args.size - 1) / mount_args.block_size) + 1;

            #[cfg(debug_assertions)]
            println!(
                "Creating block device on {} ({blocks} blocks of {} bytes)",
                mount_args.device, mount_args.block_size
            );

            device::start_disk(
                &mount_args.device,
                mount_args.gpu,
                blocks as usize,
                mount_args.block_size as usize,
            )?;
        }
    }

    Ok(())
}

fn parse_size_arg(arg: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
    let cfg = Config::new().with_binary().with_default_factor(1024 * 1024);
    let size = cfg.parse_size(arg)?;

    if size == 0 {
        Err("Disk size must be > 0")?
    }

    Ok(size)
}

fn parse_block_size_arg(arg: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
    let block_size = arg.parse::<u64>()?;

    if block_size < 512 {
        Err("Block size must be > 512")?
    }

    if block_size.count_ones() != 1 {
        Err("Block size must be a power of 2 number")?
    }

    if block_size > page_size() {
        Err(format!(
            "Block size must be less than the machine page size ({})",
            page_size()
        ))?
    }

    Ok(block_size)
}

fn page_size() -> u64 {
    unsafe { sysconf(_SC_PAGESIZE) as u64 }
}
