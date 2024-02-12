source "arm" "raspi-netboot" {
  file_urls             = ["https://downloads.raspberrypi.com/raspios_arm64/images/raspios_arm64-2023-12-06/2023-12-05-raspios-bookworm-arm64.img.xz"]
  file_checksum         = "5c54f0572d61e443a32dfa80aa8d918049814bfc70ab977f2d545eef45f1658e"
  file_checksum_type    = "sha256"
  file_target_extension = "xz"
  file_unarchive_cmd    = ["xz", "--decompress", "$ARCHIVE_PATH"]
  image_build_method    = "resize"
  image_path            = "${var.image_dir}/${var.hostname}.img"
  image_size            = "6G"
  image_type            = "dos"
  image_partitions {
    name         = "boot"
    type         = "c"
    start_sector = "8192"
    filesystem   = "fat"
    size         = "512M"
    mountpoint   = "/boot/firmware"
  }
  image_partitions {
    name         = "root"
    type         = "83"
    start_sector = "1056768"
    filesystem   = "ext4"
    size         = "0"
    mountpoint   = "/"
  }
}
