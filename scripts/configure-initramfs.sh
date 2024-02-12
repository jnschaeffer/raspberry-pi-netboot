TFTP_ROOT="/volume1/tftp/"
PI_GATEWAY="192.168.2.1"

PI_HOSTNAME=$(hostname)
PI_IP=$(ifconfig eth0 | awk '/inet /{ print $2;}')
PI_MAC=$(ifconfig eth0 | awk '/ether/{ print $2;}' | tr ':' '-')
PI_NETMASK=$(ifconfig eth0 | awk '/netmask/{ print $4;}')
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

# Update the cmdline.txt file that the pi boots with
# https://www.raspberrypi.org/documentation/configuration/cmdline-txt.md
# ip= docs can be found at
# https://git.kernel.org/pub/scm/libs/klibc/klibc.git/tree/usr/kinit/ipconfig/README.ipconfig
# However, don't put anything after static in the line below, because it completely breaks
# everything in a very non-obvious way.
sed -i -r -e \
  "s/$/ ip=${PI_IP}:::${PI_NETMASK}:${PI_HOSTNAME}:eth0:static ISCSI_INITIATOR=${ISCSI_INITIATOR_IQN} ISCSI_TARGET_NAME=${ISCSI_TARGET_IQN} ISCSI_TARGET_IP=${ISCSI_TARGET_IP} rw/g" \
  /nfs/$(hostname -s)-boot/cmdline.txt

# Update /boot/firmware/config.txt to use our new initramfs.
sed -i -r -e \
  "s@\[all\]@[all]\n\n[pi4]\ninitramfs initrd.img-$(uname -r) followkernel@" \
  /nfs/$(hostname -s)-boot/firmware/config.txt

umount /nfs/$(hostname -s)-boot
