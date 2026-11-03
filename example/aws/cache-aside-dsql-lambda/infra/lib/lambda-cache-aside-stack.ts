import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import { LambdaConstruct } from './constructs/lambda-construct';

export class LambdaCacheAsideStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    new LambdaConstruct(this, 'LambdaConstruct');

  }
}
