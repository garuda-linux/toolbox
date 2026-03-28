import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { Router } from '@angular/router';
import { Button } from 'primeng/button';
import { TranslocoDirective } from '@jsverse/transloco';
import { ElectronShellService } from '../../electron-services/electron-shell.service';
import { Card } from 'primeng/card';

@Component({
  selector: 'toolbox-setup-wizard-success',
  imports: [Button, TranslocoDirective, Card],
  templateUrl: './setup-wizard-success.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SetupWizardSuccessComponent {
  private readonly router = inject(Router);
  protected readonly shellService = inject(ElectronShellService);

  finish() {
    void this.router.navigate(['/']);
  }
}
