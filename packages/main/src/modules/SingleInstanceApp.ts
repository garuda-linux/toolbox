import type { AppModule } from '../AppModule.js';
import type { App } from 'electron';

class SingleInstanceApp implements AppModule {
  enable({ app }: { app: App }): void {
    const isSingleInstance = app.requestSingleInstanceLock();
    if (!isSingleInstance) {
      app.quit();
      throw new Error('Second instance detected - exiting');
    }
  }
}

export function disallowMultipleAppInstance(...args: ConstructorParameters<typeof SingleInstanceApp>) {
  return new SingleInstanceApp(...args);
}
