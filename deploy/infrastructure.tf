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
  ami                    = "ami-00c71bd4d220aa22a" # ubuntu-jammy-22.04-amd64-server-20240301
  instance_type          = "t2.small"
  vpc_security_group_ids = [aws_security_group.main.id]
  key_name               = aws_key_pair.aws_main_key_pair.key_name

  tags = {
    Name = "Fake stream generator"
  }

  provisioner "file" {
    source      = "./fake_stream_setup.sh"
    destination = "/tmp/setup.sh"
  }

  provisioner "remote-exec" {
    inline = [
      "sudo chmod +x /tmp/setup.sh",
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

resource "aws_security_group" "main" {
  egress = [
    {
      cidr_blocks      = ["0.0.0.0/0", ]
      description      = ""
      from_port        = 0
      ipv6_cidr_blocks = []
      prefix_list_ids  = []
      protocol         = "-1"
      security_groups  = []
      self             = false
      to_port          = 0
    }
  ]
  ingress = [
    {
      cidr_blocks      = ["0.0.0.0/0", ]
      description      = ""
      from_port        = 22
      ipv6_cidr_blocks = []
      prefix_list_ids  = []
      protocol         = "tcp"
      security_groups  = []
      self             = false
      to_port          = 22
    },
    {
      cidr_blocks      = ["0.0.0.0/0", ]
      description      = ""
      from_port        = 22
      ipv6_cidr_blocks = []
      prefix_list_ids  = []
      protocol         = "tcp"
      security_groups  = []
      self             = false
      to_port          = 80
    },
    {
      cidr_blocks      = ["0.0.0.0/0", ]
      description      = ""
      from_port        = 22
      ipv6_cidr_blocks = []
      prefix_list_ids  = []
      protocol         = "tcp"
      security_groups  = []
      self             = false
      to_port          = 9092
    }
  ]
}

resource "aws_key_pair" "aws_main_key_pair" {
  key_name   = "aws_key"
  public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDdtUGQ3lMYb/VNQWcD0Q6orz3XB1XPT6dtuS2vAtCEwmNqN0BskAJPVr3BQmA8+4tpN3A67qdJpOD6JKdkmQCN4CldbnO2hSt+DyPbTeIoOeQ/r7QuoQkLwqTPk3mZXTvQ+k1lmCJ7NDsnBHQjK+YSwnAqYou00tl8JQfI6xillqfKMKU62VoZig1jwwZVBSzLRLuUzhyRyCIt7rq5SHUsIe1rxKRaM8RdTvBbxuJuJfrmChi5ycY4k7gHZWiyqxp9WXOP7wuSZy6Gvd01jY8AtlQgjH9J3zbza/joGHBFzxPIrIEb0VY8hdWCz8WfdkwXudaKA6Lr1KWCTgJZRP9r aws-ec2-kp"
}
