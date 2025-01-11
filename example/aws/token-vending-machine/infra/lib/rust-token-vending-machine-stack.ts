import * as cdk from 'aws-cdk-lib';
import { Architecture } from 'aws-cdk-lib/aws-lambda';
import { RustFunction } from 'cargo-lambda-cdk';
import { Construct } from 'constructs';
import { FunctionConstruct } from './constructs/function-construct';


export class RustTokenVendingMachineStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const functionConstruct = new FunctionConstruct(this, 'FunctionConstruct');

  }
}
