auto eth0
allow-hotplug eth0
iface eth0 inet dhcp

iface eth0 inet6 auto
        accept_ra 2
        up ip token set ${ipv6_suffix} dev eth0