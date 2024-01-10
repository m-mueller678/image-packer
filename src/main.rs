use std::{fs, io};
use std::io::{Cursor, Seek, Write};
use std::path::{Path};
use fatfs::format_volume;
use gpt::disk::LogicalBlockSize;

fn main() {
    let disk_out=std::env::args_os().nth(2).expect("expected disk out path");
    let fat = create_fat_partition();
    let disk_size = upsize(fat.len());
    let mut disk = Cursor::new(vec![0u8; disk_size]);
    let start_offset={
        let mut gpt = gpt::GptConfig::new()
            .writable(true)
            .initialized(false)
            .logical_block_size(LogicalBlockSize::Lb512)
            .create_from_device(Box::new(&mut disk), None)
            .unwrap();
        gpt.update_partitions(Default::default()).unwrap();
        let partition_id = gpt.add_partition("boot", fat.len().try_into().unwrap(), gpt::partition_types::EFI, 0, None).unwrap();
        let partition = gpt.partitions().get(&partition_id).unwrap();
        let offset = partition.bytes_start(LogicalBlockSize::Lb512).unwrap();
        gpt.write().unwrap();
        offset
    };
    gpt::mbr::ProtectiveMBR::with_lb_size(u32::try_from(disk_size / 512).unwrap() - 1)
        .overwrite_lba0(&mut disk)
        .unwrap();
    disk.seek(io::SeekFrom::Start(start_offset)).unwrap();
    io::copy(&mut &*fat, &mut disk).unwrap();
    fs::write(disk_out,disk.into_inner()).unwrap();
}

fn upsize(s: usize) -> usize {
    (s+511)/512*512 +64 * 1024
}


fn create_fat_partition() -> Vec<u8> {
    let in_path = std::env::args_os().nth(1).expect("expected efi app path");
    let out_path = Path::new("efi/boot/bootx64.efi");
    let contents = std::fs::read(in_path).unwrap();
    let fat_size = upsize(contents.len());
    let mut fat_partition = Cursor::new(vec![0u8; fat_size]);
    format_volume(&mut fat_partition, fatfs::FormatVolumeOptions::new().volume_label(*b"EFI-system ")).unwrap();
    {
        let filesystem = fatfs::FileSystem::new(&mut fat_partition, fatfs::FsOptions::new()).unwrap();
        let root_dir = filesystem.root_dir();
        let ancestors: Vec<_> = out_path.ancestors().skip(1).collect();
        for x in ancestors.iter().rev().skip(1) {
            root_dir.create_dir(x.to_str().unwrap()).unwrap();
        }
        let mut file = root_dir.create_file(out_path.to_str().unwrap()).unwrap();
        file.truncate().unwrap();
        io::copy(&mut &*contents, &mut file).unwrap();
        file.flush().unwrap();
    }
    fat_partition.into_inner()
}