import {
  createLogCapturedEventV1,
  LogCapturedValidatorV1,
  Environment,
} from './src/telemetry/log-captured.v1';

const event = createLogCapturedEventV1(
  {
    level: 'info',
    message: 'User authentication successful',
    serviceName: 'auth-service',
    environment: Environment.Development,
    labels: {
      userId: '12345',
      method: 'oauth',
    },
  },
  'correlation-abc-123',
);

console.log('Created event:', JSON.stringify(event, null, 2));

const validator = new LogCapturedValidatorV1();
const result = validator.validate(event);

console.log('\nValidation result:', result);

const invalidEvent = createLogCapturedEventV1({
  level: 'invalid' as any,
  message: '',
  serviceName: 'test',
  environment: 'invalid-env' as any,
});

const invalidResult = validator.validate(invalidEvent);
console.log('\nInvalid event errors:', invalidResult.errors);
