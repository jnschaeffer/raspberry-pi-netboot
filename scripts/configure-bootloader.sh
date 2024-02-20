#!/bin/sh

# Step 1: Configure the bootloader. Run this on your Pi with the SD card in.

# Grab the latest bootloader from the image and generate a config from it

cp /lib/firmware/raspberrypi/bootloader-2711/stable/pieeprom-2024-01-22.bin pieeprom.bin
rpi-eeprom-config pieeprom.bin --out bootconf.txt

# Enable PXE booting, and copy over additions
echo 'BOOT_ORDER=0x21' >> bootconf.txt

# Set prefix to be the MAC address
echo 'TFTP_PREFIX=2' >> bootconf.txt

# Save the new configuration to the firmware, install it, and reboot
rpi-eeprom-config --out pieeprom-netboot.bin --config bootconf.txt pieeprom.bin
rpi-eeprom-update -d -f ./pieeprom-netboot.bin

systemctl reboot
