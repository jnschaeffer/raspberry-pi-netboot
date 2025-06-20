# raspberry-pi-netboot - Boot your Pi off iSCSI

raspberry-pi-netboot is a [Packer][packer] template for building diskless Raspberry Pi images. It's meant to be run on an ARM machine, because this is a personal project and I happen to have one. This repository also has no license because I used a lot of blog posts to put it together, so do what you want with it.

Special thanks to the following blog posts for inspiration:

* [Network Booting a Raspberry Pi 4 with an iSCSI Root via FreeNAS][sw-nas]
* [Raspberry Pi Network Boot Guide][rpi-boot]

[packer]: https://www.packer.io/
[sw-nas]: https://shawnwilsher.com/2020/05/network-booting-a-raspberry-pi-4-with-an-iscsi-root-via-freenas/
[rpi-boot]: https://warmestrobot.com/blog/2021/6/21/raspberry-pi-network-boot-guide

## Getting started

To build an image, place a `.pkrvars.hcl` file for an image in `configs/` and run `make images`. Images will be written to `images/<hostname>.img`, where `<hostname>` is the value of the `hostname` Packer template variable.

If you don't have an ARM machine to build on and want to make use of `binfmt_misc` functionality in the Packer builder, edit the Makefile to remove the `DONT_SETUP_QEMU` environment variable declaration.

Use the scripts in `scripts/` to configure the machine.

## Notes

This template and the associated scripts make some specific assumptions about the deployment environment. In particular, it assumes:

* Each machine is a Raspberry Pi 4
* Individual machines get network information from a DHCP server instead of using static IPs
* The TFTP server is running on the same subnet as the Raspberry Pi
* The DHCP server advertises the location of the TFTP server using [DHCP option 66][rfc-2132]

[net-boot]: https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#network-booting
[rfc-2132]: https://www.rfc-editor.org/rfc/rfc2132#section-9.4
