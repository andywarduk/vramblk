mod device;

fn main() {
    // TODO parms
    // TODO Lock memory

    device::start_disk("/dev/nbd0", 32768, 1024);
}
