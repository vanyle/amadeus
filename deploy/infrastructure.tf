terraform {
  required_providers {
    docker = {
      source  = "kreuzwerker/docker"
      version = "~> 3.0.1"
    }
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.16"
    }
  }
  required_version = ">= 1.2.0"
}

provider "docker" {}

provider "aws" {
  region = "eu-west-3"
}

resource "aws_instance" "app_server" {
  ami           = "ami-00c71bd4d220aa22a" # ubuntu-jammy-22.04-amd64-server-20240301
  instance_type = "t2.small"

  tags = {
    Name = "Fake stream generator"
  }

  provisioner "file" {
    source      = "./setup.sh"
    destination = "/tmp/fake_stream_setup.sh"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo chmod +x /tmp/script.sh",
      "sudo /tmp/setup.sh"
    ]
  }

  connection {
    type        = "ssh"
    host        = self.public_ip
    user        = "ubuntu"
    private_key = file("~/.ssh/aws-ec2-kp.pem")
    timeout     = "4m"
  }

}

