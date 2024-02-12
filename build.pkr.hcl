packer {}

build {
  sources = ["source.arm.raspi-netboot"]
  
  provisioner "shell" {
    environment_vars = [
      "DEBIAN_FRONTEND=noninteractive",
      "DEBCONF_NONINTERACTIVE_SEEN=true"
    ]
    inline = [
      "echo 'tzdata tzdata/Areas select US' | debconf-set-selections",
      "echo 'tzdata tzdata/Zones/US select Eastern' | debconf-set-selections",
      "rm /etc/timezone",
      "rm /etc/localtime",
      "dpkg-reconfigure -f noninteractive tzdata"
    ]
  }
  
  provisioner "shell" {
    inline = [
      "apt update",
      "apt full-upgrade -y"
    ]
  }

  provisioner "shell" {
    inline = [
      "echo 'Setting up hostname...'",
      "echo ${var.hostname} > /etc/hostname",
      "sed -i -r -e 's/(.*)raspberrypi(.*?)$/\1${var.hostname}\2/g' /etc/hosts"
    ]
  }

  provisioner "shell" {
    inline = [
      "echo 'Setting up network...'",
      "echo interface eth0 >> /etc/dhcpcd.conf",
      "echo static ip_address=${var.eth0_ipv4} >> /etc/dhcpcd.conf",
      "echo static routers=${var.eth0_gateway} >> /etc/dhcpcd.conf",
      "echo static domain_name_servers=${var.eth0_dns} >> /etc/dhcpcd.conf"
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

  provisioner "shell" {
    inline = [
      "echo 'Installing additional packages...'",
      "apt install -y initramfs-tools open-iscsi vim"
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
}
