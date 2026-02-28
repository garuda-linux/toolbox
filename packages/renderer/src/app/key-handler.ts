import type { AppComponent } from './app.component';

export async function globalKeyHandler(this: AppComponent, event: KeyboardEvent): Promise<void> {
  if (!event.ctrlKey) {
    switch (event.key) {
      case 'F1':
        void this.shellService.open('https://forum.garudalinux.org');
        break;
      case 'F2':
        void this.shellService.open('https://start.garudalinux.org');
        break;
      case 'F3':
        void this.shellService.open('https://wiki.garudalinux.org');
        break;
      case 'F4':
        this.terminalComponent().visible.set(!this.terminalComponent().visible());
        break;
      case 'F11':
        break;
    }
  } else if (event.ctrlKey) {
    switch (event.key) {
      case 'q':
        await this.requestShutdown();
        break;
    }
  }
}
