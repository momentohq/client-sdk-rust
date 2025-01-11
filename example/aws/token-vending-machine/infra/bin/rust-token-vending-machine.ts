#!/usr/bin/env node
import * as cdk from 'aws-cdk-lib';
import { RustTokenVendingMachineStack } from '../lib/rust-token-vending-machine-stack';

const app = new cdk.App();
new RustTokenVendingMachineStack(app, 'RustTokenVendingMachineStack', {});
