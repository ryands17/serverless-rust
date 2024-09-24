import * as cdk from 'aws-cdk-lib';
import * as apiGw from 'aws-cdk-lib/aws-apigatewayv2';
import * as apiGwInteg from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import * as logs from 'aws-cdk-lib/aws-logs';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as ddb from 'aws-cdk-lib/aws-dynamodb';
import { Construct } from 'constructs';
import { ApplyDestroyPolicyAspect, RustFunction } from './cdk-utils';

interface Props extends cdk.StackProps {
  /**
   * Remove all resources when the stack is deleted. Only to be used on dev and test environments
   * @default false
   */
  removeResourcesOnStackDeletion?: boolean;
}

export class PersonServiceRustStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: Props) {
    super(scope, id, props);

    const personsTable = new ddb.Table(this, 'personsTable', {
      partitionKey: { name: 'id', type: ddb.AttributeType.STRING },
      billingMode: ddb.BillingMode.PAY_PER_REQUEST,
    });

    const apiName = 'personApi';
    // API Gateway logs
    const apiGatewayLogs = new logs.LogGroup(this, `${apiName}Logs`, {
      logGroupName: `/aws/vendedlogs/${apiName}Logs`,
      retention: logs.RetentionDays.ONE_WEEK,
    });
    apiGatewayLogs.grantWrite(
      new iam.ServicePrincipal('apigateway.amazonaws.com'),
    );

    // HTTP API - API Gateway
    const api = new apiGw.HttpApi(this, apiName, {
      createDefaultStage: true,
    });

    // enable api access logging
    const stage = api.defaultStage!.node.defaultChild as apiGw.CfnStage;
    stage.accessLogSettings = {
      destinationArn: apiGatewayLogs.logGroupArn,
      format: JSON.stringify({
        requestId: '$context.requestId',
        userAgent: '$context.identity.userAgent',
        sourceIp: '$context.identity.sourceIp',
        requestTime: '$context.requestTime',
        httpMethod: '$context.httpMethod',
        path: '$context.path',
        status: '$context.status',
        responseLength: '$context.responseLength',
      }),
    };

    const addPerson = new RustFunction(this, 'add_person', {
      environment: { TABLE_NAME: personsTable.tableName },
    });
    personsTable.grantWriteData(addPerson.fn);

    api.addRoutes({
      path: '/',
      methods: [apiGw.HttpMethod.POST],
      integration: new apiGwInteg.HttpLambdaIntegration(
        'addPerson',
        addPerson.fn,
      ),
    });

    new cdk.CfnOutput(this, 'apiUrl', { value: api.apiEndpoint });

    if (props?.removeResourcesOnStackDeletion) {
      cdk.Aspects.of(this).add(new ApplyDestroyPolicyAspect());
    }
  }
}
