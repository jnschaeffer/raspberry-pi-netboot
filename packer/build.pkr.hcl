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
      "echo 'Setting up NTP...'",
      "sed -i -r -e 's/#?NTP.*?$/NTP=${var.ntpd_servers}/g' /etc/systemd/timesyncd.conf"
    ]
  }

  provisioner "shell" {
    inline = [
      "echo 'Setting up sshd...'",
      "touch /boot/ssh",
      "touch /boot/firmware/ssh",
      "echo ${var.user_password} > /boot/userconf.txt",
      "echo ${var.user_password} > /boot/firmware/userconf.txt",
      "sed -i -r -e 's/#?.*?PermitRootLogin.*?$/PermitRootLogin without-password/g' /etc/ssh/sshd_config",
      "sed -i -r -e 's/#?.*?PasswordAuthentication.*?$/PasswordAuthentication no/g' /etc/ssh/sshd_config",
      "mkdir -p /root/.ssh/",
      "chmod 700 /root/.ssh",
      "echo ${var.root_pub_key} >> /root/.ssh/authorized_keys",
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

  provisioner "file" {
    content = templatefile(
      "templates/00-eth0-prefix.pkrtpl.hcl",
      {
        ipv6_suffix = var.ipv6_suffix
      },
    )

    destination = "/etc/network/interfaces.d/00-eth0-prefix"
  }

  provisioner "shell" {
    inline = [
      "echo 'Setting up hostname...'",
      "echo ${var.hostname} > /etc/hostname",
      "sed -i -r -e 's/(.*)raspberrypi(.*?)$/\\1${var.hostname}\\2/g' /etc/hosts"
    ]
  }

  provisioner "shell" {
    inline = [
      "echo 'Setting up /etc/iscsi/iscsi.initramfs...'",
      "echo ISCSI_INITIATOR=${var.iscsi_initiator_iqn} > /etc/iscsi/iscsi.initramfs",
      "echo ISCSI_TARGET=${var.iscsi_target_iqn} >> /etc/iscsi/iscsi.initramfs",
      "echo ISCSI_TARGET_IP=${var.iscsi_target_ip} >> /etc/iscsi/iscsi.initramfs",
      "echo 'Setting up /etc/iscsi/initiatorname.iscsi...'",
      "echo InitiatorName=${var.iscsi_initiator_iqn} > /etc/iscsi/initiatorname.iscsi"
    ]
  }

  provisioner "shell" {
    inline = [
      "echo 'Rebuilding initramfs'",
      "update-initramfs -u -v"
    ]
  }
}
