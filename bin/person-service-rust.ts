#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from 'aws-cdk-lib';
import { PersonServiceRustStack } from '../lib/person-service-rust-stack';
import { environmentVars } from '../lib/cdk-utils';
import packageJson from '../package.json';

const app = new cdk.App();
new PersonServiceRustStack(app, 'PersonServiceRustStack', {
  env: { region: 'eu-west-1' },
  removeResourcesOnStackDeletion: true,
});

cdk.Tags.of(app).add('stage', environmentVars.stage);
cdk.Tags.of(app).add('version', packageJson.version);
