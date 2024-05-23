import * as path from 'path';
import * as cdk from 'aws-cdk-lib';
import {Construct} from 'constructs';
import {RustFunction} from 'cargo-lambda-cdk';

export class MomentoSimpleLambdaStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const momentoApiKeyParam = new cdk.CfnParameter(this, 'MomentoApiKey', {
      type: 'String',
      description: 'The Momento API key that will be used to read from the cache.',
      noEcho: true,
    });

    new RustFunction(this, 'MomentoSimpleRustLambda', {
      functionName: 'MomentoSimpleRustLambda',
      runtime: 'provided.al2023',
      manifestPath: path.join(__dirname, '../../lambda/momento-simple-lambda/Cargo.toml'),
      timeout: cdk.Duration.seconds(30),
      memorySize: 128,
      environment: {
        MOMENTO_API_KEY: momentoApiKeyParam.valueAsString,
      },
    });
  }
}
