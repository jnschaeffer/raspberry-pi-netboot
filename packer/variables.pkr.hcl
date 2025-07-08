variable "hostname" {
  type = string
}

variable "iscsi_initiator_iqn" {
  type = string
}

variable "iscsi_target_iqn" {
  type = string
}

variable "iscsi_target_ip" {
  type = string
}

variable "ntpd_servers" {
  type = string
}

variable "root_pub_key" {
  type = string
}

variable "user_password" {
  type = string
}

variable "image_dir" {
  type = string
}

variable ipv6_suffix {
  type = string
}