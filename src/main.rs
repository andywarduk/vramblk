#![deny(missing_docs)]

//! Create NBD GPU GPU ram device

use std::{error::Error, process::exit};

use cl::list_devices;
use clap::Parser;
use libc::{sysconf, MCL_CURRENT, MCL_FUTURE, _SC_PAGESIZE};
use parse_size::parse_size;

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

    /// Disk block size. Must be power of 2 between 512 and machine page size
    #[clap(short = 'b', long = "block-size", default_value_t = page_size(), action, value_parser = parse_block_size_arg)]
    block_size: u64,

    /// Disk size to create - eg. 1000m, 1g
    #[clap(action, value_parser = parse_size_arg)]
    size: u64,
}

fn main() {
    // Prevent this process's memory from being swapped out
    unsafe {
        libc::mlockall(MCL_CURRENT | MCL_FUTURE);
    }

    // Main process
    match process_command() {
        Ok(()) => {}
        Err(e) => {
            println!("Error: {e}");
            exit(1);
        }
    }
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
    let size = parse_size(arg)?;

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
