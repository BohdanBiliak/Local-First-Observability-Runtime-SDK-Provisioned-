import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';
import { ConfigService } from '@nestjs/config';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  const configService = app.get(ConfigService);
  const port = configService.get<number>('PORT', 3001);

  app.enableShutdownHooks();

  app.setGlobalPrefix('api/v1');

  await app.listen(port);
  console.log(`API Gateway running on http://localhost:${port}/api/v1`);
}

bootstrap();
