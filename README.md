# raspberry-pi-netboot - Boot your Pi off iSCSI

raspberry-pi-netboot contains a [Packer][packer] template for building diskless Raspberry Pi images and build scripts for provisioning them. It's meant to be run on an ARM machine, because this is a personal project and I happen to have one. This repository also has no license because I used a lot of blog posts to put it together, so do what you want with it.

Special thanks to the following blog posts for inspiration:

* [Network Booting a Raspberry Pi 4 with an iSCSI Root via FreeNAS][sw-nas]
* [Raspberry Pi Network Boot Guide][rpi-boot]

[packer]: https://www.packer.io/
[sw-nas]: https://shawnwilsher.com/2020/05/network-booting-a-raspberry-pi-4-with-an-iscsi-root-via-freenas/
[rpi-boot]: https://warmestrobot.com/blog/2021/6/21/raspberry-pi-network-boot-guide

## Getting started

To start, define your build environment in `config.env`; `config.env.example` provides an example set of values. Then, define your node configurations in `configs` using `configs/node.env.example` as an example.

To build a golden image for the nodes, run `make image`. If you don't have an ARM machine to build on and want to make use of `binfmt_misc` functionality in the Packer builder, edit the Makefile to remove the `DONT_SETUP_QEMU` environment variable declaration.

To provision the nodes, run `make provision`. This will perform the following actions for each node:

* Write bootloader files to the node's `/boot/firmware` mount point
* Create a filesystem at the node's iSCSI target and write root filesystem files to the node's `/` mount point
* Configure `/etc/fstab` and the kernel command line to boot via iSCSI
* Configure NTP to use the provided NTP server(s)
* Configure SSH to disallow passwordless auth and allow root login with the provided public key
* Configure networking to advertise an IPv6 address with the provided suffix
* Install a custom kernel (optional)

## Notes

This template and the associated scripts make some specific assumptions about the deployment environment. In particular, it assumes:

* Each machine is a Raspberry Pi 4
* Individual machines get network information from a DHCP server instead of using static IPs
* The TFTP server is running on the same subnet as the Raspberry Pi
* The DHCP server advertises the location of the TFTP server using [DHCP option 66][rfc-2132]

[net-boot]: https://www.raspberrypi.com/documentation/computers/raspberry-pi.html#network-booting
[rfc-2132]: https://www.rfc-editor.org/rfc/rfc2132#section-9.4
