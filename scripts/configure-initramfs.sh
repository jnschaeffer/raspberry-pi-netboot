#!/bin/sh

# Step 2: Configure the initramfs. Run this on your Pi after rebooting, with the SD card in.

TFTP_ROOT="/volume1/tftp/"

PI_HOSTNAME=$(hostname)
ISCSI_INITIATOR_IQN=$(cat /etc/iscsi/iscsi.initramfs | grep 'ISCSI_INITIATOR=' | cut -d '=' -f 2)
ISCSI_TARGET_IP=$(cat /etc/iscsi/iscsi.initramfs | grep 'ISCSI_TARGET_IP=' | cut -d '=' -f 2)
ISCSI_TARGET_IQN=$(cat /etc/iscsi/iscsi.initramfs | grep 'ISCSI_TARGET=' | cut -d '=' -f 2)
NFS_IP=${ISCSI_TARGET_IP}
NFS_ROOT_PATH="${TFTP_ROOT}${PI_MAC}"

# Copy over to the TFTP server, via the NFS mount.
mkdir -p /nfs/$(hostname -s)-boot
mount ${NFS_IP}:${NFS_ROOT_PATH} /nfs/$(hostname -s)-boot

rsync -a --delete /boot/firmware/ /nfs/$(hostname -s)-boot/

update-initramfs -v -k "$(uname -r)" -c -b /nfs/$(hostname -s)-boot

# Update the cmdline.txt file that the Pi boots with.
# ip= docs can be found at
# https://git.kernel.org/pub/scm/libs/klibc/klibc.git/tree/usr/kinit/ipconfig/README.ipconfig
#
# This spec uses the given hostname, the eth0 interface, and DHCP

sed -i -r -e \
  "s/$/ ip=::::${PI_HOSTNAME}:eth0:dhcp ISCSI_INITIATOR=${ISCSI_INITIATOR_IQN} ISCSI_TARGET_NAME=${ISCSI_TARGET_IQN} ISCSI_TARGET_IP=${ISCSI_TARGET_IP} rw/g" \
  /nfs/$(hostname -s)-boot/cmdline.txt

# Update /boot/firmware/config.txt to use our new initramfs

sed -i -r -e \
  "s@\[all\]@[all]\ninitramfs initrd.img-$(uname -r) followkernel@" \
  /nfs/$(hostname -s)-boot/config.txt

umount /nfs/$(hostname -s)-boot
