// oxlint-disable-next-line no-unused-vars
import { Service, signal } from '@angular/core';

@Service()
export class SplashService {
  readonly isVisible = signal(false);

  progressText = document.getElementById('progress-text');
  progressBar = document.getElementById('progress-bar');
  statusText = document.getElementById('status-text');

  /**
   * Create the splash service and set up an effect to log progress changes.
   * @param progress Progress percentage (0-100)
   * @param message Status message
   */
  updateStep(progress: number, message: string): void {
    this.adjustProgress(progress, message);
  }

  /**
   * Show the splash screen. This should be called very early in the app lifecycle, before Angular is fully loaded.
   */
  show(): void {
    this.isVisible.set(true);
  }

  /**
   * Hide the splash screen and signal the main process that the splash is complete.
   */
  async hide(): Promise<void> {
    this.isVisible.set(false);

    document.body.classList.add('angular-loaded');
    const staticSplash = document.getElementById('initial-splash');
    if (staticSplash) {
      staticSplash.style.display = 'none';
    }
  }

  /**
   * Adjust the progress bar and status text in the splash screen. Elements are plain HTML in index.html
   * not Angular components, so we manipulate them directly. This should be shown before Angular is even laoded.
   * @param value 0-100
   * @param status Status text to display
   */
  private adjustProgress(value: number, status: string): void {
    if (this.progressText) {
      this.progressText.innerText = `${value}%`;
    }
    if (this.progressBar) {
      this.progressBar.style.width = `${value}%`;
    }

    if (this.statusText) {
      this.statusText.innerText = status;
    }
  }
}
