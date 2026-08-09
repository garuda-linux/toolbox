import { Component, inject } from '@angular/core';
import { Router } from '@angular/router';
import { TranslocoDirective } from '@jsverse/transloco';
import { ElectronShellService } from '../../electron-services';
import { Card } from '@openng/optimus-ui/card';
import { ButtonDirective } from '@openng/optimus-ui/button';

@Component({
  selector: 'toolbox-setup-wizard-success',
  imports: [ButtonDirective, TranslocoDirective, Card],
  templateUrl: './setup-wizard-success.component.html',
})
export class SetupWizardSuccessComponent {
  private readonly router = inject(Router);
  protected readonly shellService = inject(ElectronShellService);

  finish() {
    void this.router.navigate(['/']);
  }
}
