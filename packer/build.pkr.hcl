packer {
  required_plugins {
    happycloud = {
      version = ">= 1.1.3"
      source = "github.com/michalfita/cross"
    }
  }
}

build {
  sources = ["source.cross.raspi-netboot"]

  provisioner "shell" {
    inline = [
      "echo 'Setting locale...'",
      "echo 'en_US.UTF-8 UTF-8' >> /etc/locale.gen",
      "dpkg-reconfigure -f noninteractive locales",
    ]
  }

  provisioner "shell" {
    environment_vars = [
      "DEBIAN_FRONTEND=noninteractive",
      "DEBCONF_NONINTERACTIVE_SEEN=true"
    ]
    inline = [
      "echo 'Setting time zone...'",
      "echo 'tzdata tzdata/Areas select US' | debconf-set-selections",
      "echo 'tzdata tzdata/Zones/US select Eastern' | debconf-set-selections",
      "rm /etc/timezone",
      "rm /etc/localtime",
      "dpkg-reconfigure -f noninteractive tzdata"
    ]
  }
  
  provisioner "shell" {
    inline = [
      "echo 'Installing packages...'",
      "apt update",
      "apt install -y initramfs-tools open-iscsi vim locales",
      "apt full-upgrade -y",
    ]
  }

  provisioner "shell" {
    inline = [
      "mkdir -p /etc/network/interfaces.d",
    ]
  }

  provisioner "shell" {
    inline = [
      "echo 'Setting up sshd...'",
      "touch /boot/ssh",
      "touch /boot/firmware/ssh",
      "sed -i -r -e 's/#?.*?PermitRootLogin.*?$/PermitRootLogin without-password/g' /etc/ssh/sshd_config",
      "sed -i -r -e 's/#?.*?PasswordAuthentication.*?$/PasswordAuthentication no/g' /etc/ssh/sshd_config",
      "mkdir -p /root/.ssh/",
      "chmod 700 /root/.ssh",
      "touch /root/.ssh/authorized_keys",
      "chmod 644 /root/.ssh/authorized_keys"
    ]
  }

  provisioner "shell" {
    inline = [
      "echo 'Disabling wifi...'",
      "echo 'dtoverlay=disable-wifi' >> /boot/firmware/config.txt"
    ]
  }

  provisioner "shell" {
    inline = [
      "echo 'Disabling bluetooth...'",
      "echo 'dtoverlay=disable-bt' >> /boot/firmware/config.txt"
    ]
  }
}
