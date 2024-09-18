import * as cdk from 'aws-cdk-lib';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as logs from 'aws-cdk-lib/aws-logs';
import * as iam from 'aws-cdk-lib/aws-iam';
import { z } from 'zod';
import { IConstruct, Construct } from 'constructs';
import path from 'path';

export const STACK_NAME = `personService`;

/**
 * Adds a removal policy of `destroy` to all resources.
 * Useful for dev and/or test environments
 */
export class ApplyDestroyPolicyAspect implements cdk.IAspect {
  public visit(node: IConstruct): void {
    if (node instanceof cdk.CfnResource) {
      node.applyRemovalPolicy(cdk.RemovalPolicy.DESTROY);
    }
  }
}

type RustFunctionProps = Omit<
  lambda.FunctionProps,
  'code' | 'handler' | 'runtime'
>;

export class RustFunction extends Construct {
  readonly fn: lambda.Function;

  constructor(scope: Construct, id: string, props?: RustFunctionProps) {
    super(scope, id);

    const name = `${STACK_NAME}-${id}`;

    // The lambda function's log group
    const logGroup = new logs.LogGroup(this, `${name}LogGroup`, {
      logGroupName: `/aws/lambda/${name}`,
      retention:
        environmentVars.stage === 'dev'
          ? logs.RetentionDays.ONE_WEEK
          : logs.RetentionDays.ONE_MONTH,
    });

    // The Lambda function's role with logging permissions
    const lambdaRole = new iam.Role(this, `${name}Role`, {
      assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
      inlinePolicies: {
        logging: new iam.PolicyDocument({
          statements: [
            new iam.PolicyStatement({
              actions: [
                'logs:CreateLogGroup',
                'logs:CreateLogStream',
                'logs:PutLogEvents',
              ],
              resources: [logGroup.logGroupArn],
            }),
          ],
        }),
      },
    });

    this.fn = new lambda.Function(this, `${name}Lambda`, {
      ...props,
      code: lambda.Code.fromAsset(
        path.join(process.cwd(), 'target', 'lambda', id),
      ),
      handler: 'bootstrap',
      runtime: lambda.Runtime.PROVIDED_AL2023,
      architecture: lambda.Architecture.ARM_64,
      logGroup,
      role: lambdaRole,
      environment: {
        ...props?.environment,
        RUST_LOG: 'info',
      },
    });
  }
}

export const environmentVars = z
  .object({
    stage: z.enum(['dev', 'test', 'prod']).default('dev'),
  })
  .parse(process.env);
