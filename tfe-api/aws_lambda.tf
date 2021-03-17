# https://learn.hashicorp.com/tutorials/terraform/lambda-api-gateway?in=terraform/aws

# 

# 

/*
Each Lambda function must have an associated IAM role which dictates what access it has to other AWS services. 
*/
resource "aws_iam_role" "iam_for_lambda" {
  name = "iam_for_lambda"

  assume_role_policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": 
 "sts:AssumeRole"
      ,
      "Principal": {
        "Service": "lambda.amazonaws.com"
      },
      "Effect": "Allow",
      "Sid": ""
    }
  ]
}
EOF
}
 
data "aws_s3_bucket_object" "bean-notification" {
  bucket = "479284709538-${var.aws_region}-aws-lambda"
  key    = "terraform-api/latest/notification.zip"
}

resource "aws_lambda_function" "bean-notification" {
  s3_bucket     = "479284709538-${var.aws_region}-aws-lambda"
  s3_key        = "terraform-api/latest/notification.zip"
  function_name = "notification"
  role          = aws_iam_role.iam_for_lambda.arn
  handler       = "notification"
  timeout       = 12

  # The filebase64sha256() function is available in Terraform 0.11.12 and later
  # For Terraform 0.11.11 and earlier, use the base64sha256() function and the file() function:
  # source_code_hash = "${base64sha256(file("lambda_function_payload.zip"))}"
  source_code_hash = base64sha256(data.aws_s3_bucket_object.bean-notification.last_modified)

  runtime = "provided"

  environment {
    variables = {
      API_KEY = var.api_key
      TFE_TOKEN = var.tfe_token
    }
  }

  # Explicitly declare dependency on EFS mount target.
  # When creating or updating Lambda functions, mount target must be in 'available' lifecycle state.
  depends_on = [
    aws_iam_role_policy_attachment.lambda_logs,
    aws_cloudwatch_log_group.bean-notification
  ]
}

# This is to optionally manage the CloudWatch Log Group for the Lambda Function.
# If skipping this resource configuration, also add "logs:CreateLogGroup" to the IAM policy below.
resource "aws_cloudwatch_log_group" "bean-notification" {
  name              = "/aws/lambda/notification" # Should be the same as function name
  retention_in_days = 14
}

# See also the following AWS managed policy: AWSLambdaBasicExecutionRole
resource "aws_iam_policy" "lambda_logging" {
  name        = "lambda_logging"
  path        = "/"
  description = "IAM policy for logging from a lambda"

  policy = <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "arn:aws:logs:*:*:*",
      "Effect": "Allow"
    }
  ]
}
EOF
}

resource "aws_iam_role_policy_attachment" "lambda_logs" {
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = aws_iam_policy.lambda_logging.arn
}
