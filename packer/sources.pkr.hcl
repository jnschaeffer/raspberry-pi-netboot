source "cross" "raspi-netboot" {
  file_urls             = ["https://downloads.raspberrypi.com/raspios_lite_arm64/images/raspios_lite_arm64-2025-05-13/2025-05-13-raspios-bookworm-arm64-lite.img.xz"]
  file_checksum         = "62d025b9bc7ca0e1facfec74ae56ac13978b6745c58177f081d39fbb8041ed45"
  file_checksum_type    = "sha256"
  file_target_extension = "xz"
  file_unarchive_cmd    = ["xz", "--decompress", "$ARCHIVE_PATH"]
  image_build_method    = "resize"
  image_path            = "${var.image_dir}/${var.hostname}.img"
  image_size            = "6G"
  image_type            = "gpt"
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
