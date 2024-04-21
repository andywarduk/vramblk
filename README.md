# vramblk - linux block device in video RAM

Creates a block linux block device in GPU video memory. The block device can then be used as swap or formatted and mounted as a filesystem.

## Building

### Debug build

```sh
cargo build
```

### Release build

```sh
cargo build --release
```

## Installing (release build)

```sh
sudo cp target/release/vramblk /usr/local/sbin
sudo chmod 700 /usr/local/sbin/vramblk
```

Check OpenCL installation by listing available GPU devices:

```sh
$ sudo /usr/local/sbin/vramblk list
Available GPU devices:
  0: NVIDIA GeForce RTX 3060 Ti, memory 7.78 GB
```

##  Running

Mount 7 GB VRAM disk on NBD device 0:

```sh
nohup sudo /usr/local/sbin/vramblk mount -d /dev/nbd0 7G &
```

### Speed test

```sh
$ dd if=/dev/nbd0 of=/dev/null bs=4096
1835008+0 records in
1835008+0 records out
7516192768 bytes (7.5 GB, 7.0 GiB) copied, 2.82887 s, 2.7 GB/s
```

### Use device as swap

Create swap on the NBD device and enable it for use at priority 0:

```sh
$ sudo mkswap /dev/nbd0
Setting up swapspace version 1, size = 7 GiB (7516188672 bytes)
no label, UUID=5f07e348-0059-4f76-8b0c-0a4e3d54b23b
$ sudo swapon -p 0 /dev/nbd0
```

Check swaps:

```sh
$ swapon
NAME             TYPE      SIZE   USED PRIO
/dev/nbd0        partition   7G     0B    0
```

Remove swap device

```sh
sudo swapoff /dev/nbd0
```

### Use device as disk

Create an EXT filesystem on the device:

```sh
$ mke2fs /dev/nbd0
mke2fs 1.47.0 (5-Feb-2023)
Discarding device blocks: done                            
Creating filesystem with 1835008 4k blocks and 458752 inodes
Filesystem UUID: 91db3000-4480-4933-bfef-4c592d9b1a5c
Superblock backups stored on blocks: 
        32768, 98304, 163840, 229376, 294912, 819200, 884736, 1605632

Allocating group tables: done                            
Writing inode tables: done                            
Writing superblocks and filesystem accounting information: done 
```

Mount the disk

```sh
$ sudo mount /dev/nbd0 /mnt/tmp
$ ls -l /mnt/tmp
drwx------ root root 16 KB Sun Apr 21 10:30:38 2024 lost+found
```

Unmount the disk

```sh
sudo umount /mnt/tmp
```

## Credits

The vblk rust crate by Thomas Bénéteau made this really easy :) [crates.io](https://crates.io/crates/vblk), [github](https://github.com/TomCrypto/vblk)

The opencl3 crate by Volker Mische and Ken Barker [crates.io](https://crates.io/crates/opencl3)
