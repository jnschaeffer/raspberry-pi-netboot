#!/bin/sh

# Note: This is less a shell script and more a collection of snippets to use. Set environment variables
# accordingly.

# Set ISCSI_TARGET_IP and ISCSI_TARGET_IQN and log into the iSCSI target

sudo iscsiadm \
  --mode discovery \
  --type sendtargets \
  --portal ${ISCSI_TARGET_IP}
sudo iscsiadm \
  --mode node \
  --targetname ${ISCSI_TARGET_IQN} \
  --portal ${ISCSI_TARGET_IP} \
  --login

# Find the iSCSI device, set ISCSI_DEVICE to the device path, and create a partition

sudo parted ${ISCSI_DEVICE} mklabel gpt
sudo parted --align optimal ${ISCSI_DEVICE} mkpart primary ext4 0% 100%

# Set ISCSI_ROOT_PARTITION to the root partition and make a filesystem

sudo mkfs.ext4 ${ISCSI_ROOT_PARTITION}
ISCSI_ROOT_DIR="$(mktemp -d)"
sudo mount "${ISCSI_ROOT_PARTITION}" ${ISCSI_ROOT_DIR}

# Set IMAGE_PATH to the generated image path and mount the root partition in the generated image

CREATION_OUTPUT=$(sudo partx --add -v ${IMAGE_PATH})
LOOP_DEVICE=$(echo ${CREATION_OUTPUT} | sed -E 's@.*(/dev/loop[0-9]+).*@\1@')
IMAGE_ROOT_DIR="$(mktemp -d)"
sudo mount "${LOOP_DEVICE}p2" ${IMAGE_ROOT_DIR}

# Copy the root dir contents to the iSCSI root
sudo rsync -a --info=progress2 "${IMAGE_ROOT_DIR}/" ${ISCSI_ROOT_DIR}

# Unmount the loop device and clean up the dangling partitions
sudo umount ${IMAGE_ROOT_DIR}
sudo partx --delete -v ${LOOP_DEVICE}

# Set PI_MAC to the MAC address of the server and TFTP_ROOT to the TFTP root, then update /etc/fstab
# so that /boot points to the NFS directory and / points to the iSCSI target

# Update /boot

NFS_IP=$(cat ${ISCSI_ROOT_DIR}/etc/iscsi/iscsi.initramfs | grep 'ISCSI_TARGET_IP=' | cut -d '=' -f 2)
NFS_ROOT_PATH="${TFTP_ROOT}${PI_MAC}"
sudo sed -i -r -E \
  "s@.*/boot/firmware +.*@${NFS_IP}:${NFS_ROOT_PATH} /boot/firmware nfs defaults,vers=4.1,proto=tcp 0 0@" \
  ${ISCSI_ROOT_DIR}/etc/fstab

# Update /
ISCSI_ROOT_PARTUUID=$(sudo blkid | grep ${ISCSI_ROOT_PARTITION} | tr ' ' '\n' | grep PARTUUID | cut -d '=' -f 2 | tr -d '"')
sudo sed -i -r -E \
  "s@.*/ +.*@PARTUUID=${ISCSI_ROOT_PARTUUID} / ext4 _netdev,noatime 0 1@" \
  ${ISCSI_ROOT_DIR}/etc/fstab
  
# Log out of the iSCSI target

sudo umount ${ISCSI_ROOT_DIR}
sudo iscsiadm --m node -T ${ISCSI_TARGET_IQN} --portal ${ISCSI_TARGET_IP}:3260 -u
sudo iscsiadm -m node -o delete -T ${ISCSI_TARGET_IQN} --portal ${ISCSI_TARGET_IP}:3260

# Mount the NFS directory and update the kernel command line with the new PARTUUID

NFS_BOOT_DIR=$(mktemp -d)

sudo mount ${NFS_IP}:${NFS_ROOT_PATH} $NFS_BOOT_DIR
sudo sed -i -r -E \
     "s/root=PARTUUID=[0-9a-f-]+/root=PARTUUID=${ISCSI_ROOT_PARTUUID}/" \
     ${NFS_BOOT_DIR}/cmdline.txt

# Clean up the NFS mount

sudo umount ${NFS_BOOT_DIR}
