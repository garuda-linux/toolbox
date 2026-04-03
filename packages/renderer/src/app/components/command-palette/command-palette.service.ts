import { Injectable, signal } from '@angular/core';
import type { CommandPaletteAction } from '../../interfaces';

@Injectable({
  providedIn: 'root',
})
export class CommandPaletteService {
  private actions = signal<CommandPaletteAction[]>([]);

  registerActions(...newActions: CommandPaletteAction[]): void {
    this.actions.update((current) => {
      const filtered = current.filter((existing) => !newActions.some((a) => a.id === existing.id));
      return [...filtered, ...newActions];
    });
  }

  unregisterActions(...ids: string[]): void {
    this.actions.update((current) => current.filter((a) => !ids.includes(a.id)));
  }

  getActions(): CommandPaletteAction[] {
    return this.actions();
  }

  clearActions(): void {
    this.actions.set([]);
  }
}
