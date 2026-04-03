import type { CommandPaletteAction } from '../../interfaces';

const APP_ACTIONS_BASE: Omit<CommandPaletteAction, 'command'>[] = [
  {
    id: 'help-forum',
    label: 'menu.help.getHelpForum',
    icon: 'pi pi-comments',
    keywords: ['help', 'forum', 'support', 'community'],
    category: 'help',
  },
  {
    id: 'help-wiki',
    label: 'menu.help.getHelpWiki',
    icon: 'pi pi-book',
    keywords: ['help', 'wiki', 'documentation', 'docs'],
    category: 'help',
  },
  {
    id: 'help-arch-wiki',
    label: 'menu.help.getHelpArchWiki',
    icon: 'pi pi-book',
    keywords: ['help', 'arch', 'wiki', 'documentation', 'archwiki'],
    category: 'help',
  },
  {
    id: 'help-status',
    label: 'menu.help.garudaStatus',
    icon: 'pi pi-info-circle',
    keywords: ['help', 'status', 'infra', 'infrastructure'],
    category: 'help',
  },
];

const NAVIGATION_ACTIONS: CommandPaletteAction[] = [
  {
    id: 'nav-settings',
    label: 'menu.file.preferences',
    icon: 'pi pi-cog',
    keywords: ['settings', 'preferences', 'config'],
    category: 'navigation',
    routerLink: '/settings',
  },
  {
    id: 'nav-diagnostics',
    label: 'menu.modules.diagnostics',
    icon: 'pi pi-info-circle',
    keywords: ['diagnostics', 'logs', 'debug'],
    category: 'navigation',
    routerLink: '/diagnostics',
  },
  {
    id: 'nav-packages',
    label: 'menu.modules.packages',
    icon: 'pi pi-box',
    keywords: ['packages', 'software', 'apps'],
    category: 'navigation',
    routerLink: '/packages',
  },
];

export function registerAppCommandPaletteActions(
  registerFn: (...actions: CommandPaletteAction[]) => void,
  getCommand: (id: string) => (() => void | Promise<void>) | undefined,
): void {
  const actionsWithCommands: CommandPaletteAction[] = [];

  for (const action of APP_ACTIONS_BASE) {
    const command = getCommand(action.id);
    if (command) {
      actionsWithCommands.push({ ...action, command });
    }
  }

  registerFn(...actionsWithCommands, ...NAVIGATION_ACTIONS);
}
