import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { Router } from '@angular/router';
import { TranslocoDirective } from '@jsverse/transloco';
import { ElectronShellService } from '../../electron-services';
import { Card } from 'primeng/card';
import { ButtonDirective } from 'primeng/button';
import { ExternalLink } from '@primeicons/angular/external-link';
import { Check } from '@primeicons/angular/check';

@Component({
  selector: 'toolbox-setup-wizard-success',
  imports: [ButtonDirective, TranslocoDirective, Card, ExternalLink, Check],
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
