# nas-pi - Pi stuff for your NAS

nas-pi is a [Packer][packer] template for building diskless Raspberry Pi images. It's meant to be run on an ARM machine, because this is a personal project and I happen to have one.

Special thanks to the following blog posts for inspiration:

* [Network Booting a Raspberry Pi 4 with an iSCSI Root via FreeNAS][sw-nas]
* [Raspberry Pi Network Boot Guide][rpi-boot]

[packer]: https://www.packer.io/
[sw-nas]: https://shawnwilsher.com/2020/05/network-booting-a-raspberry-pi-4-with-an-iscsi-root-via-freenas/
[rpi-boot]: https://warmestrobot.com/blog/2021/6/21/raspberry-pi-network-boot-guide

## Getting started

To build an image, place a `.pkrvars.hcl` file for an image in `configs/` and run `make images`. Images will be written to `images/<hostname>.img`, where `<hostname>` is the value of the `hostname` Packer template variable.

If you don't have an ARM machine to build on and want to make use of `binfmt_misc` functionality in the Packer builder, edit the Makefile to remove the `DONT_SETUP_QEMU` environment variable declaration.
