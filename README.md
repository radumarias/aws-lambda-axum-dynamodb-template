# aws-lambda-axum-dynamodb

Simple aws lambda with axum, DynamoDB, API Gateway and CloudWatch

# Install

In AWS IAM crate a Policy `MyResourceGroupsFullAccess` with this

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "resource-groups:CreateGroup",
                "resource-groups:DeleteGroup",
                "resource-groups:GetGroup",
                "resource-groups:GetGroupQuery",
                "resource-groups:GetTags",
                "resource-groups:Tag",
                "resource-groups:Untag",
                "resource-groups:UpdateGroup",
                "resource-groups:UpdateGroupQuery",
                "resource-groups:ListGroupResources",
                "resource-groups:ListGroups"
            ],
            "Resource": "*"
        }
    ]
}
```

Assign it to a AWS user.

Make sure that user have these permissions:
- AWSLambda_FullAccess
- AWSCloudFormationFullAccess
- AWSCodeDeployRoleForCloudFormation
- AmazonS3FullAccess
- IAMFullAccess
- CloudWatchApplicationInsightsFullAccess
- CloudWatchLambdaInsightsExecutionRolePolicy
- AmazonAPIGatewayAdministrator
- AmazonAPIGatewayInvokeFullAccess
- MyResourceGroupsFullAccess

And you created and `Access keys` and added to `$HOME/.aws/credentials`

You should have something like this in `$HOME/.aws/config`
```
[default]
region = us-east-1
output = json
```

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
yay -S aws-sam-cli --no-confirm
```

# Build

```bash
cd rust && cargo build && cd ..
sam build --beta-features
```

# Run locally

```bash
cd rust && cargo run
```

You can view the API specs in [api.yml](https://raw.githubusercontent.com/radumarias/aws-lamda-axum-dynamodb/main/api.yml)

Some test calls:

```bash
curl -X POST http://localhost:3000/v1/upload/eb1438c0-57f3-4fc7-8fe0-b83e664954f1 -d '{"hash": "123"}' -H "Content-Type: application/json"
curl -X POST http://localhost:3000/v1/upload/eb1438c0-57f3-4fc7-8fe0-b83e664954f2 -d '{"hash": "456"}' -H "Content-Type: application/json"

curl -X GET http://localhost:3000/v1/results/eb1438c0-57f3-4fc7-8fe0-b83e664954ff\?page\=1\&per_page\=10
curl -X GET http://localhost:3000/v1/path\?page\=1\&per_page\=10\&src\=eb1438c0-57f3-4fc7-8fe0-b83e664954f1\&dst\=eb1438c0-57f3-4fc7-8fe0-b83e664954f2
```

# Deploy

```bash
sam deploy --guided
```

You can use something like this:

```bash
Setting default arguments for 'sam deploy'
=========================================
Stack Name [sam-app]: rust
AWS Region [us-east-1]:
#Shows you resources changes to be deployed and require a 'Y' to initiate deploy
Confirm changes before deploy [y/N]:
#SAM needs permission to be able to create roles to connect to the resources in your template
Allow SAM CLI IAM role creation [Y/n]:
#Preserves the state of previously provisioned resources when an operation fails
Disable rollback [y/N]:
AxumFunction has no authentication. Is this okay? [y/N]: y
Save arguments to configuration file [Y/n]:
SAM configuration file [samconfig.toml]:
SAM configuration environment [default]:
```

- goto AWS API Gateway
- select your app with the name you gave at deploy
- in the lef menu select `API: <name-of-your-app>(something)`
- in the `Stages` section you have the `Invoke URL`, use that to make API calls
