#!/usr/bin/env node

import * as cdk from 'aws-cdk-lib';
import { LambdaCacheAsideStack } from '../lib/lambda-cache-aside-stack';

const app = new cdk.App();
new LambdaCacheAsideStack(app, 'LambdaCacheAsideDSQLStack', {
    env: {
        region: "us-east-1"
    }
});
