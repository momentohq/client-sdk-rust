import { SecretValue } from "aws-cdk-lib";
import { Architecture } from "aws-cdk-lib/aws-lambda";
import { Secret } from "aws-cdk-lib/aws-secretsmanager";
import { RustFunction } from "cargo-lambda-cdk";
import { Construct } from "constructs";
import path = require("path");

export class FunctionConstruct extends Construct {

  constructor(scope: Construct, id: string) {
    super(scope, id);

    const secret = new Secret(scope, 'MomentoKeySecret', {
      secretName: 'MomentoApiKeySecret',
      secretObjectValue: {
        momentoSecret: SecretValue.unsafePlainText(process.env.MOMENTO_API_KEY!)
      }
    });

    const vendingMachine = new RustFunction(scope, 'TokenVendingMachineFunction', {
      architecture: Architecture.ARM_64,
      functionName: "momento-token-vending-machine",
      manifestPath: path.join(__dirname, `../../../lambdas/Cargo.toml`),
      memorySize: 256,
      environment: {
        RUST_LOG: 'info',
      },
    })

    secret.grantRead(vendingMachine);

  }

}
