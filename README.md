# raspberry-pi-netboot - Boot your Pi off iSCSI

raspberry-pi-netboot contains a [Packer][packer] template for building diskless Raspberry Pi images and build scripts for provisioning them. It's meant to be run on an ARM machine, because this is a personal project and I happen to have one. This repository also has no license because I used a lot of blog posts to put it together, so do what you want with it.

Special thanks to the following blog posts for inspiration:

* [Network Booting a Raspberry Pi 4 with an iSCSI Root via FreeNAS][sw-nas]
* [Raspberry Pi Network Boot Guide][rpi-boot]

[packer]: https://www.packer.io/
[sw-nas]: https://shawnwilsher.com/2020/05/network-booting-a-raspberry-pi-4-with-an-iscsi-root-via-freenas/
[rpi-boot]: https://warmestrobot.com/blog/2021/6/21/raspberry-pi-network-boot-guide

## Getting started

To start, build a golden image for the nodes by running `make image`. This will write a Raspberry Pi OS image to `images/raspios.img`. If you don't have an ARM machine to build on and want to make use of `binfmt_misc` functionality in the Packer builder, edit the Makefile to remove the `DONT_SETUP_QEMU` environment variable declaration.

Once you have an image, define your build configuration in a JSON file in `configs/workspace.json`. This file defines fleet-level settings like OS image and network storage locations. Then, define, your instance configurations as JSON files in the `configs/instances` directory. These files define instance-level settings like instance ID, iSCSI IQNs, MAC address, and authentication information. For more information about these values, see `provision/src/config.rs`.

To provision the nodes, run `make provision`. This will perform the following actions for each node:

* Write bootloader files to the node's `/boot/firmware` mount point
* Create a filesystem at the node's iSCSI target and write root filesystem files to the node's `/` mount point
* Configure `/etc/fstab` and the kernel command line to boot via iSCSI
* Configure SSH to disallow passwordless auth and allow root login with the provided public key

The exact build steps are located in `provision/src/steps.rs`. The graph defining build steps is located in `provision/src/lib.rs`.

## Notes

This provisioning flow makes some assumptions about the deployment environment. In particular, it assumes:

* Each machine is a Raspberry Pi 4
* Each machine is configured as a [network boot client][net-boot]
* A DHCP server manages host configuration
* The DHCP server advertises the location of an NTP server using [DHCP option 42][rfc-2132-42]
* The DHCP server advertises the location of the TFTP server using [DHCP option 66][rfc-2132-66]
* The TFTP server is running on the same subnet as the Raspberry Pi

[net-boot]: https://www.raspberrypi.com/documentation/computers/remote-access.html#configure-a-network-boot-client
[rfc-2132-42]: https://www.rfc-editor.org/rfc/rfc2132#section-8.3
[rfc-2132-66]: https://www.rfc-editor.org/rfc/rfc2132#section-9.4
