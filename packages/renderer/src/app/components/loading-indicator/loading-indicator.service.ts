import { computed, Service, signal } from '@angular/core';
import { Logger } from '../../logging/logging';

@Service()
export class LoadingService {
  private readonly loadingRefCounter = signal<number>(0);
  private readonly logger = Logger.getInstance();
  readonly loading = computed(() => this.loadingRefCounter() > 0);

  loadingOn(): void {
    this.logger.trace('Loading on');
    this.loadingRefCounter.update((prev) => prev + 1);
  }

  loadingOff(): void {
    this.logger.trace('Loading off');
    this.loadingRefCounter.update((prev) => prev - 1);
  }
}
