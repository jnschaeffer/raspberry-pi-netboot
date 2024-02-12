#!/bin/sh

TFTP_IP="192.168.2.1"

cp /lib/firmware/raspberrypi/bootloader-2711/stable/pieeprom-2024-01-22.bin pieeprom.bin
rpi-eeprom-config pieeprom.bin --out bootconf.txt

# Enable PXE booting, and copy over additions.
echo 'BOOT_ORDER=0x21' >> bootconf.txt

# Set prefix to be the MAC address, like normal PXE things.
echo 'TFTP_PREFIX=2' >> bootconf.txt

# Set the TFTP IP address (no need to mess with DHCP settings).
echo "TFTP_IP=${TFTP_IP}" >> bootconf.txt

# Set this device's IP address.
PI_IP=$(ifconfig eth0 | awk '/inet /{ print $2;}')
PI_NETMASK=$(ifconfig eth0 | awk '/netmask/{ print $4;}')
echo "CLIENT_IP=${PI_IP}
SUBNET=${PI_NETMASK}" >> bootconf.txt

# Save the new configuration to the firmware, and install it (requires a reboot).
rpi-eeprom-config --out pieeprom-netboot.bin --config bootconf.txt pieeprom.bin
rpi-eeprom-update -d -f ./pieeprom-netboot.bin

systemctl reboot
