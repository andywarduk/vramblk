use std::io::Result;

use vblk::{mount, BlockDevice};

struct RamDisk {
    blocks: usize,
    block_size: usize,
    memory: Vec<u8>,
}

impl RamDisk {
    fn new(blocks: usize, block_size: usize) -> Self {
        Self {
            blocks,
            block_size,
            memory: vec![0; blocks * block_size]
        }
    }

    fn mount(&mut self, nbd_device: &str) -> Result<()> {
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
                }).expect("Failed to install terminate handler");

                Ok(())
            })
        }
    }
}

impl BlockDevice for RamDisk {
    fn read(&mut self, offset: u64, bytes: &mut [u8]) -> Result<()> {
        println!("read request offset {} len {}", offset, bytes.len());

        bytes.copy_from_slice(&self.memory[offset as usize..offset as usize + bytes.len()]);

        Ok(())
    }

    fn write(&mut self, offset: u64, bytes: &[u8]) -> Result<()> {
        println!("write request offset {} len {}", offset, bytes.len());

        self.memory[offset as usize..offset as usize + bytes.len()].copy_from_slice(bytes);

        Ok(())
    }

    fn unmount(&mut self) {
        println!("ramdisk unmounted!");
    }

    fn flush(&mut self) -> Result<()> {
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

pub fn start_disk(nbd_device: &str, blocks: usize, block_size: usize) {
    // Create the block device
    let mut blkdev = RamDisk::new(blocks, block_size);

    // Mount the block device
    match blkdev.mount(nbd_device) {
        Ok(()) => {},
        Err(e) => eprintln!("Error mounting block device on {nbd_device}: {e}"),
    }
}

