# Terraform

We use terraform to simply deploy our project in a reproducable manner.

To get started, install terraform using [this guide](https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli)

Then, run `terraform init`.

Then, `cp .example.env .env` and add your AWS access key inside the `.env` file.

Make sure your file is valid using `terraform validate`

Apply the changes you made to AWS using `terraform apply`

You can see how an existing instance is doing using:

```bash
terraform state show aws_instance.app_server
```
