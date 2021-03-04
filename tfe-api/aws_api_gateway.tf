/*
API Gateway's name reflects its original purpose as a public-facing frontend for REST APIs, 
but it was later extended with features that make it easy to expose an entire web application based on AWS Lambda. 
These later features will be used in this tutorial. The term "REST API" is thus used loosely here, 
since API Gateway is serving as a generic HTTP frontend rather than necessarily serving an API.

Create a new file aws_api_gateway.tf in the same directory as our lambda.tf from the previous step. First, configure the root "REST API" object, as follows:  
*/
resource "aws_api_gateway_rest_api" "bean-notification" {
  name        = "BeanNotificationApi"
  description = "AWS Serverless Application to handle TFE Api"
}

/*
The "REST API" is the container for all of the other API Gateway objects we will create.
All incoming requests to API Gateway must match with a configured resource and method in order to be handled. 
Append the following to define a single proxy resource:
*/
resource "aws_api_gateway_resource" "bean-notification-proxy" {
   rest_api_id = aws_api_gateway_rest_api.bean-notification.id
   parent_id   = aws_api_gateway_rest_api.bean-notification.root_resource_id
   path_part   = "{proxy+}"
}
resource "aws_api_gateway_method" "bean-notification-proxy" {
   rest_api_id   = aws_api_gateway_rest_api.bean-notification.id
   resource_id   = aws_api_gateway_resource.bean-notification-proxy.id
   http_method   = "ANY"
   authorization = "NONE"
}

/*
The special path_part value "{proxy+}" activates proxy behavior, which means that this resource will match any request path. 
Similarly, the aws_api_gateway_method block uses a http_method of "ANY", which allows any request method to be used. 
Taken together, this means that all incoming requests will match this resource.
Each method on an API gateway resource has an integration which specifies where incoming requests are routed. 
Add the following configuration to specify that requests to this method should be sent to the Lambda function defined earlier:
*/
resource "aws_api_gateway_integration" "bean-notification" {
   rest_api_id = aws_api_gateway_rest_api.bean-notification.id
   resource_id = aws_api_gateway_method.bean-notification-proxy.resource_id
   http_method = aws_api_gateway_method.bean-notification-proxy.http_method

   integration_http_method = "POST"
   type                    = "AWS_PROXY"
   uri                     = aws_lambda_function.bean-notification.invoke_arn
}

/*
The AWS_PROXY integration type causes API gateway to call into the API of another AWS service. 
In this case, it will call the AWS Lambda API to create an "invocation" of the Lambda function.
Unfortunately the proxy resource cannot match an empty path at the root of the API. 
To handle that, a similar configuration must be applied to the root resource that is built in to the REST API object:
*/
resource "aws_api_gateway_method" "bean-notification-proxy_root" {
   rest_api_id   = aws_api_gateway_rest_api.bean-notification.id
   resource_id   = aws_api_gateway_rest_api.bean-notification.root_resource_id
   http_method   = "ANY"
   authorization = "NONE"
}

resource "aws_api_gateway_integration" "bean-notification_root" {
   rest_api_id = aws_api_gateway_rest_api.bean-notification.id
   resource_id = aws_api_gateway_method.bean-notification-proxy_root.resource_id
   http_method = aws_api_gateway_method.bean-notification-proxy_root.http_method

   integration_http_method = "POST"
   type                    = "AWS_PROXY"
   uri                     = aws_lambda_function.bean-notification.invoke_arn
}

/*
Finally, you need to create an API Gateway "deployment" in order to activate the configuration and expose the API at a URL that can be used for testing:
*/
resource "aws_api_gateway_deployment" "bean-notification" {
   depends_on = [
     aws_api_gateway_integration.bean-notification,
     aws_api_gateway_integration.bean-notification_root,
   ]

   rest_api_id = aws_api_gateway_rest_api.bean-notification.id
   stage_name  = "master" # tfe_variables.tf
}

/*
After the creation steps are complete, the new objects will be visible in the API Gateway console.
The integration with the Lambda function is not functional yet
because API Gateway does not have the necessary access to invoke the function.
The next step will address this, making the application fully-functional.

»Allowing API Gateway to Access Lambda
By default any two AWS services have no access to one another,
until access is explicitly granted. For Lambda functions, access is granted using the aws_lambda_permission resource,
which should be added to the lambda.tf file created in an earlier step:
*/
resource "aws_lambda_permission" "apigw" {
   statement_id  = "AllowAPIGatewayInvoke"
   action        = "lambda:InvokeFunction"
   function_name = aws_lambda_function.bean-notification.function_name
   principal     = "apigateway.amazonaws.com"

   # The "/*/*" portion grants access from any method on any resource
   # within the API Gateway REST API.
   source_arn = "${aws_api_gateway_rest_api.bean-notification.execution_arn}/*/*"
}

/*
In order to test the created API you will need to access its test URL.
To make this easier to access, add the following output to api_gateway.tf:
*/
output "base_url" {
   value = aws_api_gateway_deployment.bean-notification.invoke_url
}
