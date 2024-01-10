tool to create uefi-bootable disk images.

# usage
```
cargo run -- application.efi disk.img
```
creates a gpt partitioned disk image `disk.img` with a single efi system partition.
The partition contains a single file at `efi/boot/bootx64.efi` with the contents of `application.efi`.