import { NgModule } from '@angular/core';
import { RouterModule, type Routes } from '@angular/router';

export const routes: Routes = [
  {
    title: 'Garuda Toolbox',
    path: '',
    loadComponent: () => import('./components/home/home.component').then((m) => m.HomeComponent),
  },
  {
    title: 'Maintenance',
    path: 'maintenance',
    loadComponent: () => import('./components/maintenance/maintenance.component').then((m) => m.MaintenanceComponent),
  },
  {
    title: 'System tools',
    path: 'system-tools',
    loadComponent: () => import('./components/system-tools/system-tools.component').then((m) => m.SystemToolsComponent),
  },
  {
    title: 'Gaming apps',
    path: 'gaming',
    loadComponent: () => import('./components/gaming/gaming.component').then((m) => m.GamingComponent),
  },
  {
    title: 'Diagnostics',
    path: 'diagnostics',
    loadComponent: () => import('./components/diagnostics/diagnostics.component').then((m) => m.DiagnosticsComponent),
  },
  {
    title: 'Settings',
    path: 'settings',
    loadComponent: () => import('./components/settings/settings.component').then((m) => m.SettingsComponent),
  },
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule],
})
// eslint-disable-next-line @typescript-eslint/no-extraneous-class
export class AppRoutingModule {}
