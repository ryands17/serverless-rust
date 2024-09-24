# Person microservice (Rust)

A simple serverless lambda built in Rust using the CDK

## Prerequisites

- Node LTS v20 (or above)
- PNPM
- Cargo
- [Cargo watch](https://crates.io/crates/cargo-watch) (for running builds in watch mode)

## Useful commands

- `pnpm dev` run CDK and build rust lambdas in watch mode
- `pnpm build` compile typescript to js
- `pnpm watch` watch for changes and compile
- `pnpm test` perform the jest unit tests
- `pnpm cdk deploy` deploy this stack to your default AWS account/region
- `pnpm cdk diff` compare deployed stack with current state
- `pnpm cdk synth` emits the synthesized CloudFormation template
