import { ChangeDetectionStrategy, Component, computed, inject, type OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router, type UrlTree } from '@angular/router';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { MessageToastService } from '@garudalinux/core';
import { ConfirmationService } from '@openng/optimus-ui/api';
import { ButtonDirective } from '@openng/optimus-ui/button';
import { Checkbox } from '@openng/optimus-ui/checkbox';
import { DialogModule } from '@openng/optimus-ui/dialog';
import { IconField } from '@openng/optimus-ui/iconfield';
import { InputIcon } from '@openng/optimus-ui/inputicon';
import { InputNumber } from '@openng/optimus-ui/inputnumber';
import { InputText } from '@openng/optimus-ui/inputtext';
import { MultiSelectModule } from '@openng/optimus-ui/multiselect';
import { Password } from '@openng/optimus-ui/password';
import { type Popover, PopoverModule } from '@openng/optimus-ui/popover';
import { Select } from '@openng/optimus-ui/select';
import { TableModule } from '@openng/optimus-ui/table';
import { Tab, TabList, TabPanel, TabPanels, Tabs } from '@openng/optimus-ui/tabs';
import { Tag } from '@openng/optimus-ui/tag';
import { ToggleButton } from '@openng/optimus-ui/togglebutton';
import { Tooltip } from '@openng/optimus-ui/tooltip';
import type { CommandPaletteAction } from '../../interfaces';
import { Logger } from '../../logging/logging';
import { CommandPaletteService } from '../command-palette/command-palette.service';
import { UserManagerService } from './user-manager.service';
import type { ManagedGroup, ManagedUser } from './types';

@Component({
  selector: 'toolbox-user-manager',
  imports: [
    ButtonDirective,
    Checkbox,
    DialogModule,
    FormsModule,
    IconField,
    InputIcon,
    InputNumber,
    InputText,
    MultiSelectModule,
    Password,
    PopoverModule,
    Select,
    TableModule,
    Tab,
    TabList,
    TabPanel,
    TabPanels,
    Tabs,
    Tag,
    ToggleButton,
    Tooltip,
    TranslocoDirective,
  ],
  templateUrl: './user-manager.component.html',
  styleUrl: './user-manager.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UserManagerComponent implements OnInit {
  private readonly commandPaletteService = inject(CommandPaletteService);
  private readonly confirmationService = inject(ConfirmationService);
  private readonly logger = Logger.getInstance();
  private readonly messageToastService = inject(MessageToastService);
  private readonly router = inject(Router);
  private readonly translocoService = inject(TranslocoService);
  private readonly userManagerService = inject(UserManagerService);

  readonly tabIndex = signal<number>(0);
  readonly loading = signal<boolean>(true);
  readonly showAllUsers = signal<boolean>(false);

  readonly shells = [
    '/bin/bash',
    '/bin/sh',
    '/bin/zsh',
    '/bin/fish',
    '/usr/bin/nushell',
    '/usr/bin/tcsh',
    '/usr/bin/python',
    '/bin/false',
    '/sbin/nologin',
  ];

  readonly users = computed(() =>
    [...this.userManagerService.currentUsers().values()]
      .filter((user) => this.showAllUsers() || !user.system)
      .sort((a, b) => a.username.localeCompare(b.username)),
  );
  readonly groups = computed(() =>
    [...this.userManagerService.currentGroups().values()].sort((a, b) => a.name.localeCompare(b.name)),
  );
  readonly groupNames = computed(() => [...this.userManagerService.currentGroups().keys()].sort());

  activeUser = signal<ManagedUser | null>(null);
  activeGroup = signal<ManagedGroup | null>(null);

  userDialogVisible = signal<boolean>(false);
  userDialogMode = signal<'add' | 'edit'>('add');
  userDialogUsername = signal<string>('');
  userDialogFullName = signal<string>('');
  userDialogHome = signal<string>('');
  userDialogShell = signal<string>('/bin/bash');
  userDialogUid = signal<number | null>(null);
  userDialogCreateHome = signal<boolean>(true);
  userDialogSystem = signal<boolean>(false);
  userDialogGroups = signal<string[]>([]);
  userDialogPassword = signal<string>('');
  userDialogPasswordConfirm = signal<string>('');

  addGroupVisible = signal<boolean>(false);
  newGroupName = signal<string>('');

  groupMembersVisible = signal<boolean>(false);
  groupMembersUser = signal<ManagedUser | null>(null);
  groupMembersSelected = signal<string[]>([]);

  passwordVisible = signal<boolean>(false);
  passwordUser = signal<ManagedUser | null>(null);
  passwordValue = signal<string>('');
  passwordConfirm = signal<string>('');

  ngOnInit(): void {
    void this.userManagerService.update().finally(() => this.loading.set(false));
    this.registerCommandPaletteActions();

    this.checkRoute();

    this.router.events.subscribe(() => {
      const currentUrl: UrlTree = this.router.parseUrl(this.router.url);
      if (currentUrl.fragment) {
        this.navigateToTab(currentUrl.fragment);
      }
    });
  }

  private checkRoute(): void {
    const url: UrlTree = this.router.parseUrl(this.router.url);
    this.navigateToTab(url.fragment ?? '');
  }

  private registerCommandPaletteActions(): void {
    const actions: CommandPaletteAction[] = [
      {
        id: 'user-manager',
        label: 'menu.modules.userManager',
        icon: 'pi pi-users',
        keywords: ['user', 'users', 'group', 'groups', 'account', 'membership', 'password'],
        category: 'app',
        routerLink: 'system-tools',
        hash: 'users',
      },
    ];
    this.commandPaletteService.registerActions(...actions);
  }

  navigateToTab(fragment: string): void {
    switch (fragment) {
      case 'groups':
        this.tabIndex.set(1);
        break;
      case 'users':
      default:
        this.tabIndex.set(0);
    }
  }

  navigate(fragment: string): void {
    void this.router.navigate([], { fragment, info: { disableViewTransition: true } });
  }

  private tr(key: string, params?: Record<string, string>): string {
    return this.translocoService.translate(key, params);
  }

  openAddUserDialog(): void {
    this.userDialogMode.set('add');
    this.userDialogUsername.set('');
    this.userDialogFullName.set('');
    this.userDialogHome.set('');
    this.userDialogShell.set('/bin/bash');
    this.userDialogUid.set(null);
    this.userDialogCreateHome.set(true);
    this.userDialogSystem.set(false);
    this.userDialogGroups.set([]);
    this.userDialogPassword.set('');
    this.userDialogPasswordConfirm.set('');
    this.userDialogVisible.set(true);
  }

  openEditUserDialog(user: ManagedUser): void {
    this.userDialogMode.set('edit');
    this.userDialogUsername.set(user.username);
    this.userDialogFullName.set(user.fullName);
    this.userDialogHome.set(user.home);
    this.userDialogShell.set(user.shell);
    this.userDialogUid.set(null);
    this.userDialogCreateHome.set(false);
    this.userDialogSystem.set(user.system);
    this.userDialogGroups.set([...user.groups]);
    this.userDialogPassword.set('');
    this.userDialogPasswordConfirm.set('');
    this.userDialogVisible.set(true);
  }

  onUsernameChange(): void {
    if (this.userDialogMode() === 'add' && !this.userDialogHome()) {
      this.userDialogHome.set(`/home/${this.userDialogUsername()}`);
    }
  }

  saveUser(): void {
    const username = this.userDialogUsername().trim();
    if (!username) {
      this.messageToastService.error(this.tr('userManager.error'), this.tr('userManager.enterUsername'));
      return;
    }

    const change = {
      username,
      fullName: this.userDialogFullName().trim() || undefined,
      home: this.userDialogHome().trim() || undefined,
      shell: this.userDialogShell(),
      uid: this.userDialogUid() ?? undefined,
      createHome: this.userDialogCreateHome(),
      system: this.userDialogSystem(),
    };

    const desiredGroups = this.userDialogGroups();

    if (this.userDialogMode() === 'add') {
      if (this.userManagerService.currentUsers().has(username)) {
        this.messageToastService.error(this.tr('userManager.error'), this.tr('userManager.userExists'));
        return;
      }
      const password = this.userDialogPassword();
      if (password !== this.userDialogPasswordConfirm()) {
        this.messageToastService.error(this.tr('userManager.error'), this.tr('userManager.passwordMismatch'));
        return;
      }
      this.userManagerService.addUser({ ...change, password: password || undefined });
      for (const group of desiredGroups) {
        this.userManagerService.setMembership(username, group, true);
      }
    } else {
      this.userManagerService.modifyUser(change);
      const current = this.userManagerService.currentUsers().get(username);
      const currentGroups = current?.groups ?? [];
      for (const group of new Set([...currentGroups, ...desiredGroups])) {
        if (currentGroups.includes(group) !== desiredGroups.includes(group)) {
          this.userManagerService.setMembership(username, group, desiredGroups.includes(group));
        }
      }
    }

    this.userDialogVisible.set(false);
  }

  openGroupMembersDialog(user: ManagedUser): void {
    this.groupMembersUser.set(user);
    this.groupMembersSelected.set([...user.groups]);
    this.groupMembersVisible.set(true);
  }

  saveGroupMembers(): void {
    const user = this.groupMembersUser();
    if (!user) return;

    const desired = this.groupMembersSelected();
    const current = user.groups;
    for (const group of new Set([...current, ...desired])) {
      if (current.includes(group) !== desired.includes(group)) {
        this.userManagerService.setMembership(user.username, group, desired.includes(group));
      }
    }
    this.groupMembersVisible.set(false);
  }

  toggleGroupMembership(group: string): void {
    this.groupMembersSelected.update((selected) => {
      const next = [...selected];
      const index = next.indexOf(group);
      if (index !== -1) {
        next.splice(index, 1);
      } else {
        next.push(group);
      }
      return next.sort();
    });
  }

  toggleAllGroupMemberships(checked: boolean): void {
    this.groupMembersSelected.set(checked ? [...this.groupNames()] : []);
  }

  gidOf(group: string): number | undefined {
    return this.userManagerService.currentGroups().get(group)?.gid;
  }

  memberCountOf(group: string): number {
    return this.userManagerService.currentGroups().get(group)?.members.length ?? 0;
  }

  openAddGroupDialog(): void {
    this.newGroupName.set('');
    this.addGroupVisible.set(true);
  }

  saveGroup(): void {
    const name = this.newGroupName().trim();
    if (!name) {
      this.messageToastService.error(this.tr('userManager.error'), this.tr('userManager.enterGroupName'));
      return;
    }
    if (this.userManagerService.currentGroups().has(name)) {
      this.messageToastService.error(this.tr('userManager.error'), this.tr('userManager.groupExists'));
      return;
    }
    this.userManagerService.addGroup(name);
    this.addGroupVisible.set(false);
  }

  confirmRemoveUser(user: ManagedUser): void {
    this.confirmationService.confirm({
      message: this.tr('userManager.confirmRemoveUser'),
      header: this.tr('userManager.removeUser'),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonProps: {
        severity: 'danger',
        label: this.tr('userManager.removeUser'),
      },
      rejectButtonProps: {
        severity: 'secondary',
        label: this.tr('confirmation.reject'),
      },
      accept: () => {
        this.userManagerService.removeUser(user.username);
        this.logger.debug(`Queued removal of user ${user.username}`);
      },
    });
  }

  confirmRemoveGroup(group: ManagedGroup): void {
    this.confirmationService.confirm({
      message: this.tr('userManager.confirmRemoveGroup'),
      header: this.tr('userManager.removeGroup'),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonProps: {
        severity: 'danger',
        label: this.tr('userManager.removeGroup'),
      },
      rejectButtonProps: {
        severity: 'secondary',
        label: this.tr('confirmation.reject'),
      },
      accept: () => {
        this.userManagerService.removeGroup(group.name);
        this.logger.debug(`Queued removal of group ${group.name}`);
      },
    });
  }

  lockUser(user: ManagedUser): void {
    this.userManagerService.queueLock(user.username, true);
  }

  unlockUser(user: ManagedUser): void {
    this.userManagerService.queueLock(user.username, false);
  }

  openPasswordDialog(user: ManagedUser): void {
    this.passwordUser.set(user);
    this.passwordValue.set('');
    this.passwordConfirm.set('');
    this.passwordVisible.set(true);
  }

  savePassword(): void {
    const user = this.passwordUser();
    if (!user) return;

    const password = this.passwordValue();
    const confirm = this.passwordConfirm();
    if (!password) {
      this.messageToastService.error(this.tr('userManager.error'), this.tr('userManager.enterPassword'));
      return;
    }
    if (password !== confirm) {
      this.messageToastService.error(this.tr('userManager.error'), this.tr('userManager.passwordMismatch'));
      return;
    }

    this.userManagerService.queuePassword(user.username, password);
    this.passwordVisible.set(false);
  }

  pendingUserState(user: ManagedUser): string | null {
    const change = this.userManagerService.wantedUsers().get(user.username);
    if (change) {
      if (change.action === 'add') return 'add';
      if (change.action === 'remove') return 'remove';
      if (change.action === 'modify') return 'modify';
    }
    const lock = this.userManagerService.pendingLock(user.username);
    if (lock) return lock;
    if (this.userManagerService.pendingPassword(user.username)) return 'password';
    if (this.userManagerService.wantedMemberships().has(user.username)) return 'membership';
    return null;
  }

  pendingGroupState(group: ManagedGroup): string | null {
    const wanted = this.userManagerService.wantedGroups().get(group.name);
    if (wanted !== undefined) return wanted ? 'add' : 'remove';
    return null;
  }

  userPendingCount(user: ManagedUser): number {
    let count = 0;
    if (this.userManagerService.wantedUsers().has(user.username)) count++;
    if (this.userManagerService.wantedMemberships().has(user.username)) count++;
    if (this.userManagerService.pendingLock(user.username)) count++;
    if (this.userManagerService.pendingPassword(user.username)) count++;
    return count;
  }

  clearUserChanges(user: ManagedUser): void {
    this.userManagerService.clearUserChanges(user.username);
  }

  clearGroupChanges(group: ManagedGroup): void {
    this.userManagerService.clearGroupChanges(group.name);
  }

  isLockQueued(user: ManagedUser): boolean {
    return this.userManagerService.pendingLock(user.username) === 'lock';
  }

  openPopover($event: MouseEvent, op: Popover, user: ManagedUser): void {
    this.activeUser.set(user);
    op.toggle($event);
  }

  openGroupPopover($event: MouseEvent, op: Popover, group: ManagedGroup): void {
    this.activeGroup.set(group);
    op.toggle($event);
  }
}
