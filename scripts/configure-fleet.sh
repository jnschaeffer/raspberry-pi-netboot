#!/bin/bash

set -eux

# Set up cleanup

function confirm () {
    echo -e "\033[1m$1\033[0m"
    echo -en '\033[1mtype yes to continue: \033[0m'
    read REPLY
    if [[ "$REPLY" != 'yes' ]]; then
        echo -e "\033[1maborting\033[0m"
        exit 1
    fi
}

CLEANUP_CMDS=()

function cleanup_add () {
    CLEANUP_CMDS[${#CLEANUP_CMDS[@]}]="$1"
}

function cleanup () {
    for i in $(seq "$((${#CLEANUP_CMDS[@]} - 1))" -1 0); do
        # Try to run the cleanup command, but keep going if it fails
        ${CLEANUP_CMDS[${i}]} || true
    done
}

trap cleanup SIGINT SIGTERM EXIT

# Set up a mount workspace, create directories

workspace_dir="$(pwd)/$(mktemp -d raspberry-pi.XXXXX)"

img_boot_mount="${workspace_dir}/mounts/boot"
img_rootfs_mount="${workspace_dir}/mounts/rootfs"

nfs_mount="${workspace_dir}/mounts/nfs"

node_rootfs_mount="${workspace_dir}/mounts/node/rootfs"

mkdir -p "${workspace_dir}" "${img_boot_mount}" "${img_rootfs_mount}" "${nfs_mount}" "${node_rootfs_mount}"

cleanup_add "rmdir ${img_boot_mount} ${img_rootfs_mount} ${nfs_mount} ${node_rootfs_mount} ${workspace_dir}/mounts/node ${workspace_dir}/mounts ${workspace_dir}"

# Mount the image as a loop device
partx_out="$(sudo partx --add -v ${IMAGE_PATH})"
loop_device="$(echo ${partx_out} | sed -E 's@.*(/dev/loop[0-9]+).*@\1@')"

cleanup_add "sudo partx --delete -v ${loop_device}"

sudo mount -o ro "${loop_device}p1" ${img_boot_mount}

cleanup_add "sudo umount ${img_boot_mount}"

sudo mount -o ro "${loop_device}p2" ${img_rootfs_mount}

cleanup_add "sudo umount ${img_rootfs_mount}"

# Mount the NFS volume

sudo mount ${NFS_IP}:${NFS_ROOT_PATH} $nfs_mount

cleanup_add "sudo umount ${nfs_mount}"

# Discover iSCSI targets

sudo iscsiadm \
  --mode discovery \
  --type sendtargets \
  --portal ${ISCSI_TARGET_IP}

# For each node...

for config_file in $(find "${CONFIG_DIR}" -name "*.env"); do
    echo "Configuring node in ${config_file}"

    # Source the env file

    set -o allexport
    source "${config_file}"
    set +o allexport

    # Copy the contents of /boot/firmware to the NFS mount at MAC address

    node_boot_dir="${nfs_mount}/${NODE_MAC_ADDRESS}"
    
    confirm "about to overwrite ALL data at ${node_boot_dir}"
    
    sudo rsync --delete -a --info=progress2 "${img_boot_mount}/" "${node_boot_dir}"

    sync

    # Mount the iSCSI LUN

    sudo iscsiadm \
         --mode node \
         --targetname ${NODE_ISCSI_TARGET_IQN} \
         --portal ${ISCSI_TARGET_IP} \
         --login

    cleanup_add "sudo iscsiadm --m node -T ${NODE_ISCSI_TARGET_IQN} --portal ${ISCSI_TARGET_IP} --logout"

    # Format the LUN with a new filesystem

    iscsi_device="/dev/disk/by-path/ip-${ISCSI_TARGET_IP}:3260-iscsi-${NODE_ISCSI_TARGET_IQN}-lun-1"

    confirm "about to wipe ALL data on ${iscsi_device}"

    sudo parted "${iscsi_device}" mklabel gpt
    sudo parted --align optimal "${iscsi_device}" mkpart primary ext4 0% 100%

    iscsi_root_part="${iscsi_device}-part1"

    sudo mkfs.ext4 "${iscsi_root_part}"
    
    # Mount the new root partition

    sudo mount "${iscsi_root_part}" "${node_rootfs_mount}"

    # We'll explicitly do this step anyway but if anything fails, we'll still
    # unmount the disk
    cleanup_add "sudo umount ${node_rootfs_mount}"

    # Copy the contents of / to the LUN

    sudo rsync -a --info=progress2 "${img_rootfs_mount}/" ${node_rootfs_mount}

    sync

    # Configure /etc/fstab

    sudo sed -i -r -E \
         "s@.*/boot(/firmware)? +.*@${NFS_IP}:${NFS_ROOT_PATH}/${NODE_MAC_ADDRESS} /boot/firmware nfs defaults,vers=4.1,proto=tcp 0 0@" \
         ${node_rootfs_mount}/etc/fstab

    iscsi_root_partuuid=$(sudo lsblk -n -o PARTUUID ${iscsi_root_part})

    if [[ "${iscsi_root_partuuid}" == "" ]]; then
        echo "iSCSI root PARTUUID not found"
        exit 1
    fi

    sudo sed -i -r -E \
         "s@.*/ +.*@PARTUUID=${iscsi_root_partuuid} / ext4 _netdev,noatime 0 1@" \
         ${node_rootfs_mount}/etc/fstab

    # Configure kernel command line

    sudo sed -i -r -E \
         "s/root=PARTUUID=[0-9a-f-]+/root=PARTUUID=${iscsi_root_partuuid}/" \
         ${node_boot_dir}/cmdline.txt

    sudo sed -i -r -e \
        "s/$/ ip=::::${NODE_HOSTNAME}:eth0:dhcp ISCSI_INITIATOR=${NODE_ISCSI_INITIATOR_IQN} ISCSI_TARGET_NAME=${NODE_ISCSI_TARGET_IQN} ISCSI_TARGET_IP=${ISCSI_TARGET_IP} rw/g" \
        ${node_boot_dir}/cmdline.txt

    # Configure NTP

    sudo sed -i -r -e "s/#?NTP.*?$/NTP=${NTPD_SERVER}/g" "${node_rootfs_mount}/etc/systemd/timesyncd.conf"

    # Configure SSH

    echo "${NODE_USER_PASSWORD}" | sudo tee "${node_boot_dir}/userconf.txt" "${node_rootfs_mount}/boot/userconf.txt" >/dev/null

    echo "${NODE_ROOT_PUB_KEY}" | sudo tee "${node_rootfs_mount}/root/.ssh/authorized_keys" >/dev/null
    
    # Configure hostname

    echo "${NODE_HOSTNAME}" | sudo tee "${node_rootfs_mount}/etc/hostname" >/dev/null

    sudo sed -i -r -e "s/(.*)raspberrypi(.*?)$/\\1${NODE_HOSTNAME}\\2/g" "${node_rootfs_mount}/etc/hosts"

    # Configure network interfaces

    interfaces_dir="${node_rootfs_mount}/etc/network/interfaces.d"
    
    sudo mkdir -p "${interfaces_dir}"

    sudo tee "${interfaces_dir}/00-eth0-prefix" <<EOF
auto eth0
allow-hotplug eth0
iface eth0 inet dhcp

iface eth0 inet6 auto
        accept_ra 2
        up ip token set ${NODE_IPV6_SUFFIX} dev eth0
EOF

    # Add custom kernel if one exists
    if [[ "${KERNEL_REPO}" != "" ]]; then
        (
            cd "${KERNEL_REPO}"
            sudo env PATH="$PATH" make -j12 INSTALL_MOD_PATH="${node_rootfs_mount}" modules_install
            sudo cp arch/arm64/boot/Image "${node_boot_dir}"/kernel8.img
            sudo cp arch/arm64/boot/dts/broadcom/*.dtb "${node_boot_dir}"
            sudo cp arch/arm64/boot/dts/overlays/*.dtb* "${node_boot_dir}/overlays/"
            sudo cp arch/arm64/boot/dts/overlays/README "${node_boot_dir}/overlays/"

            # Sync once again (cp should do this but why risk it?)

            sync
        )
    fi

    # Explicitly unmount the iSCSI LUN and move on

    sudo umount "${node_rootfs_mount}"

done
