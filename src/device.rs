use std::{error::Error, ptr};

use opencl3::{
    command_queue::CommandQueue,
    context::Context,
    memory::{Buffer, CL_MEM_READ_WRITE},
    types::CL_BLOCKING,
};
use vblk::{mount, BlockDevice};

use crate::cl::initialise_cl;

struct VRamDisk {
    blocks: usize,
    block_size: usize,
    _cl_context: Context,
    cl_queue: CommandQueue,
    memory: Buffer<u8>,
}

impl VRamDisk {
    fn new(gpu: Option<u16>, blocks: usize, block_size: usize) -> Result<Self, Box<dyn Error>> {
        // Initialise CL
        let (context, queue) = initialise_cl(gpu)?;

        // Allocate a buffer on the CL device
        #[cfg(debug_assertions)]
        println!("Creating CL buffer");

        let memory = unsafe {
            Buffer::<u8>::create(
                &context,
                CL_MEM_READ_WRITE,
                blocks * block_size,
                ptr::null_mut(),
            )
        }
        .map_err(|e| format!("Failed to create CL buffer: {e}"))?;

        // Make sure we can read from it
        let mut read_test = [0u8; 1];

        let _event =
            unsafe { queue.enqueue_read_buffer(&memory, CL_BLOCKING, 0, &mut read_test, &[]) }
                .map_err(|e| format!("Failed to create CL buffer: {e}"))?;

        // Return new struct
        Ok(Self {
            blocks,
            block_size,
            _cl_context: context,
            cl_queue: queue,
            memory,
        })
    }

    fn mount(&mut self, nbd_device: &str) -> std::io::Result<()> {
        unsafe {
            // Mount the device
            mount(self, nbd_device, |device| {
                // Set ctrl-c / termination handler
                ctrlc::set_handler(move || {
                    // Unmount the device
                    match device.unmount() {
                        Ok(()) => (),
                        Err(e) => eprintln!("Failed to unmount device: {e}"),
                    }
                })
                .expect("Failed to install terminate handler");

                Ok(())
            })
        }
    }
}

impl BlockDevice for VRamDisk {
    fn read(&mut self, offset: u64, bytes: &mut [u8]) -> std::io::Result<()> {
        #[cfg(debug_assertions)]
        println!("Read request: offset {} len {}", offset, bytes.len());

        let _event = unsafe {
            self.cl_queue.enqueue_read_buffer(
                &self.memory,
                CL_BLOCKING,
                offset as usize,
                bytes,
                &[],
            )
        }
        .map_err(|cl_err| {
            println!("Read error: {}", cl_err);
            std::io::Error::new(std::io::ErrorKind::Other, cl_err)
        })?;

        Ok(())
    }

    fn write(&mut self, offset: u64, bytes: &[u8]) -> std::io::Result<()> {
        #[cfg(debug_assertions)]
        println!("Write request: offset {} len {}", offset, bytes.len());

        let _event = unsafe {
            self.cl_queue.enqueue_write_buffer(
                &mut self.memory,
                CL_BLOCKING,
                offset as usize,
                bytes,
                &[],
            )
        }
        .map_err(|cl_err| {
            println!("Write error: {}", cl_err);
            std::io::Error::new(std::io::ErrorKind::Other, cl_err)
        })?;

        Ok(())
    }

    fn unmount(&mut self) {
        #[cfg(debug_assertions)]
        println!("device unmounted");
    }

    fn flush(&mut self) -> std::io::Result<()> {
        #[cfg(debug_assertions)]
        println!("flush request");

        Ok(())
    }

    fn block_size(&self) -> u32 {
        self.block_size as u32
    }

    fn blocks(&self) -> u64 {
        self.blocks as u64
    }
}

pub fn start_disk(
    nbd_device: &str,
    gpu: Option<u16>,
    blocks: usize,
    block_size: usize,
) -> Result<(), Box<dyn Error>> {
    // Create the block device
    let mut blkdev = VRamDisk::new(gpu, blocks, block_size)?;

    // Mount the block device
    blkdev.mount(nbd_device)?;

    Ok(())
}
