#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { PersonServiceRustStack } from '../lib/person-service-rust-stack';

const app = new cdk.App();
new PersonServiceRustStack(app, 'PersonServiceRustStack', {
  env: { region: 'eu-west-1' },
});
