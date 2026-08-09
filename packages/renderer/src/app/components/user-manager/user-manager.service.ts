import { effect, inject, Service, signal, untracked } from '@angular/core';
import { Logger } from '../../logging/logging';
import { TaskManagerService } from '../task-manager/task-manager.service';
import type { ManagedGroup, ManagedUser, UserChange } from './types';

@Service()
export class UserManagerService {
  private readonly logger = Logger.getInstance();
  private readonly taskManagerService = inject(TaskManagerService);

  /**
   * The current state of all system users, keyed by username.
   */
  readonly currentUsers = signal<Map<string, ManagedUser>>(new Map());

  /**
   * The current state of all system groups, keyed by group name.
   */
  readonly currentGroups = signal<Map<string, ManagedGroup>>(new Map());

  /**
   * Pending user operations, keyed by username. Each entry describes what should
   * happen to the user once the pending operations are applied.
   */
  readonly wantedUsers = signal<Map<string, UserChange>>(new Map());

  /**
   * Pending group operations, keyed by group name. `true` means the group should
   * be added, `false` means it should be removed.
   */
  readonly wantedGroups = signal<Map<string, boolean>>(new Map());

  /**
   * Pending group memberships, keyed by username. Each entry maps a group name to
   * the desired membership state (`true` = member, `false` = not a member).
   */
  readonly wantedMemberships = signal<Map<string, Map<string, boolean>>>(new Map());

  constructor() {
    effect(() => {
      if (!this.taskManagerService.running()) {
        void untracked(async () => await this.update());
      }
    });

    effect(() => {
      this.pruneWantedState();
    });

    effect(() => {
      this.generateTasks();
    });
  }

  /**
   * Prune the wanted state once the current state matches what was requested.
   * Mirrors the behavior of the os-interact service so finished operations
   * disappear from the pending queue automatically.
   */
  private pruneWantedState(): void {
    const currentUsers = this.currentUsers();
    const currentGroups = this.currentGroups();

    this.wantedUsers.update((wanted) => {
      const pruned = new Map(wanted);
      for (const [username, change] of wanted) {
        const current = currentUsers.get(username);
        if (change.action === 'add' && current) {
          pruned.delete(username);
          continue;
        }
        if (change.action === 'remove' && !current) {
          pruned.delete(username);
          continue;
        }
        if (change.action === 'modify') {
          if (!current) {
            pruned.delete(username);
            continue;
          }
          let dirty = false;
          if (change.shell !== undefined && change.shell !== current.shell) dirty = true;
          if (change.fullName !== undefined && change.fullName !== current.fullName) dirty = true;
          if (change.home !== undefined && change.home !== current.home) dirty = true;
          if (!dirty) pruned.delete(username);
        }
      }
      return pruned;
    });

    this.wantedGroups.update((wanted) => {
      const pruned = new Map(wanted);
      for (const [group, shouldExist] of wanted) {
        if (shouldExist === currentGroups.has(group)) {
          pruned.delete(group);
        }
      }
      return pruned;
    });

    this.wantedMemberships.update((wanted) => {
      const pruned = new Map(wanted);
      for (const [username, groups] of wanted) {
        const userMap = new Map(groups);
        const current = currentUsers.get(username);
        for (const [group, desired] of groups) {
          const isMember = current?.groups.includes(group) ?? false;
          if (desired === isMember) {
            userMap.delete(group);
          }
        }
        if (userMap.size === 0) {
          pruned.delete(username);
        } else {
          pruned.set(username, userMap);
        }
      }
      return pruned;
    });
  }

  /**
   * Generate the tasks to be executed based on the wanted state and the current state.
   */
  private generateTasks(): void {
    const currentUsers = this.currentUsers();
    const currentGroups = this.currentGroups();

    const tasks: Parameters<typeof this.taskManagerService.scheduleTask>[0][] = [];

    const pendingAdds = new Set(
      [...this.wantedUsers()].filter(([, change]) => change.action === 'add').map(([username]) => username),
    );
    const pendingRemoves = new Set(
      [...this.wantedUsers()].filter(([, change]) => change.action === 'remove').map(([username]) => username),
    );

    for (const [username, change] of this.wantedUsers()) {
      if (change.action === 'remove') {
        const script = `userdel -r '${username}' 2>/dev/null || userdel '${username}'`;
        tasks.push(
          this.taskManagerService.createTask(
            30,
            `user-manager-remove-${username}`,
            true,
            'userManager.task.removeUser',
            'pi pi-user-minus',
            script,
          ),
        );
      } else if (change.action === 'add') {
        let command = 'useradd';
        if (change.system) command += ' --system';
        if (change.createHome) command += ' --create-home';
        if (change.home) command += ` --home-dir '${change.home}'`;
        if (change.shell) command += ` --shell '${change.shell}'`;
        if (change.uid !== undefined) command += ` --uid ${change.uid}`;
        if (change.fullName) command += ` --comment '${change.fullName}'`;
        const groups = this.desiredGroups(username);
        if (groups.length > 0) command += ` --groups '${groups.join(',')}'`;
        command += ` ${username}`;
        if (change.password) {
          command += ` && printf '%s\\n' ${this.shellQuote(`${username}:${change.password}`)} | chpasswd`;
        }
        tasks.push(
          this.taskManagerService.createTask(
            30,
            `user-manager-add-${username}`,
            true,
            'userManager.task.addUser',
            'pi pi-user-plus',
            command,
          ),
        );
      } else if (change.action === 'modify') {
        let command = 'usermod';
        if (change.shell) command += ` --shell '${change.shell}'`;
        if (change.fullName) command += ` --comment '${change.fullName}'`;
        if (change.home) command += ` --home '${change.home}'`;
        if (command !== 'usermod') {
          tasks.push(
            this.taskManagerService.createTask(
              30,
              `user-manager-modify-${username}`,
              true,
              'userManager.task.modifyUser',
              'pi pi-pencil',
              `${command} ${username}`,
            ),
          );
        }
      }
    }

    for (const [group, shouldExist] of this.wantedGroups()) {
      if (shouldExist) {
        tasks.push(
          this.taskManagerService.createTask(
            31,
            `user-manager-group-add-${group}`,
            true,
            'userManager.task.addGroup',
            'pi pi-users',
            `groupadd '${group}'`,
          ),
        );
      } else {
        tasks.push(
          this.taskManagerService.createTask(
            31,
            `user-manager-group-remove-${group}`,
            true,
            'userManager.task.removeGroup',
            'pi pi-users',
            `groupdel '${group}'`,
          ),
        );
      }
    }

    for (const [username, groups] of this.wantedMemberships()) {
      if (pendingAdds.has(username) || pendingRemoves.has(username)) continue;
      const userExists = currentUsers.has(username);
      for (const [group, desired] of groups) {
        if (!desired && !currentGroups.has(group)) continue;
        tasks.push(
          this.taskManagerService.createTask(
            32,
            `user-manager-membership-${username}-${group}`,
            true,
            desired ? 'userManager.task.addMembership' : 'userManager.task.removeMembership',
            'pi pi-users',
            `gpasswd ${desired ? '-a' : '-d'} '${username}' '${group}'`,
          ),
        );
      }
      if (!userExists) {
        this.logger.warn(`Membership change for unknown user ${username}`);
      }
    }

    const desiredIds = new Set(tasks.map((task) => task.id));
    untracked(() => {
      for (const task of this.taskManagerService.tasks()) {
        if (desiredIds.has(task.id)) {
          this.taskManagerService.removeTask(task);
        }
      }
    });

    for (const task of tasks) {
      this.taskManagerService.scheduleTask(task);
    }
  }

  /**
   * Compute the desired group set of a user, based on the current state and the
   * pending membership changes. Used when creating a new user so the groups can
   * be passed directly to `useradd`.
   * @param username The username to compute the groups for.
   */
  private desiredGroups(username: string): string[] {
    const current = this.currentUsers().get(username);
    const wanted = this.wantedMemberships().get(username);
    const groups = new Set(current?.groups ?? []);
    if (wanted) {
      for (const [group, desired] of wanted) {
        if (desired) {
          groups.add(group);
        } else {
          groups.delete(group);
        }
      }
    }
    return [...groups].sort();
  }

  /**
   * Refresh the current state of users and groups from the system.
   */
  async update(): Promise<void> {
    const [passwdResult, groupResult, shadowResult] = await Promise.all([
      this.taskManagerService.executeAndWaitBash('getent passwd'),
      this.taskManagerService.executeAndWaitBash('getent group'),
      this.taskManagerService.executeAndWaitBash(`pkexec cat /etc/shadow`),
    ]);

    if (passwdResult.code !== 0) {
      this.logger.warn(`getent passwd failed: ${passwdResult.stderr}`);
    }
    if (groupResult.code !== 0) {
      this.logger.warn(`getent group failed: ${groupResult.stderr}`);
    }
    if (shadowResult.code !== 0) {
      this.logger.warn(`Failed to read shadow: ${shadowResult.stderr}`);
    }

    const lockedUsers = new Set<string>();
    for (const line of shadowResult.stdout.split('\n')) {
      if (!line.trim()) continue;
      const [username, hash] = line.split(':');
      if (hash?.startsWith('!')) lockedUsers.add(username);
    }

    const users = new Map<string, ManagedUser>();
    for (const line of passwdResult.stdout.split('\n')) {
      if (!line.trim()) continue;
      const parts = line.split(':');
      if (parts.length < 7) continue;
      const [username, , uidStr, gidStr, gecos, home, shell] = parts;
      const uid = parseInt(uidStr, 10);
      if (!Number.isNaN(uid)) {
        users.set(username, {
          username,
          uid,
          gid: parseInt(gidStr, 10),
          fullName: gecos ?? '',
          home,
          shell,
          locked: lockedUsers.has(username),
          system: uid < 1000,
          groups: [],
        });
      }
    }

    const groups = new Map<string, ManagedGroup>();
    for (const line of groupResult.stdout.split('\n')) {
      if (!line.trim()) continue;
      const parts = line.split(':');
      if (parts.length < 4) continue;
      const [name, , gidStr, membersStr] = parts;
      const members = membersStr ? membersStr.split(',') : [];
      groups.set(name, { name, gid: parseInt(gidStr, 10), members });
    }

    for (const [username, user] of users) {
      const userGroups = [...groups.values()]
        .filter((group) => group.members.includes(username) || group.gid === user.gid)
        .map((group) => group.name)
        .sort();
      users.set(username, { ...user, groups: userGroups });
    }

    this.currentUsers.set(users);
    this.currentGroups.set(groups);
    this.logger.debug(`Refreshed ${users.size} users and ${groups.size} groups`);
  }

  /**
   * Queue a new user for creation.
   * @param change The desired state of the new user.
   */
  addUser(change: Omit<UserChange, 'action'>): void {
    this.wantedUsers.update((wanted) => {
      const next = new Map(wanted);
      next.set(change.username, { ...change, action: 'add' });
      return next;
    });
  }

  /**
   * Queue a user for modification.
   * @param change The desired modifications.
   */
  modifyUser(change: Omit<UserChange, 'action'>): void {
    this.wantedUsers.update((wanted) => {
      const next = new Map(wanted);
      next.set(change.username, { ...change, action: 'modify' });
      return next;
    });
  }

  /**
   * Queue a user for removal.
   * @param username The username to remove.
   */
  removeUser(username: string): void {
    this.wantedUsers.update((wanted) => {
      const next = new Map(wanted);
      next.set(username, { action: 'remove', username });
      return next;
    });
  }

  /**
   * Set the desired membership of a user in a group.
   * @param username The username.
   * @param group The group name.
   * @param wanted Whether the user should be a member.
   */
  setMembership(username: string, group: string, wanted: boolean): void {
    this.wantedMemberships.update((memberships) => {
      const next = new Map(memberships);
      const userMap = new Map(next.get(username) ?? []);
      const isMember = this.currentUsers().get(username)?.groups.includes(group) ?? false;
      if (wanted === isMember) {
        userMap.delete(group);
      } else {
        userMap.set(group, wanted);
      }
      if (userMap.size === 0) {
        next.delete(username);
      } else {
        next.set(username, userMap);
      }
      return next;
    });
  }

  /**
   * Queue a group for creation.
   * @param group The group name.
   */
  addGroup(group: string): void {
    this.wantedGroups.update((wanted) => new Map(wanted).set(group, true));
  }

  /**
   * Queue a group for removal.
   * @param group The group name.
   */
  removeGroup(group: string): void {
    this.wantedGroups.update((wanted) => new Map(wanted).set(group, false));
  }

  /**
   * Queue a lock or unlock action for a user. These are one-shot operations that
   * run immediately once the pending operations are applied.
   * @param username The username.
   * @param lock Whether to lock (true) or unlock (false) the account.
   */
  queueLock(username: string, lock: boolean): void {
    const id = `user-manager-${lock ? 'lock' : 'unlock'}-${username}`;
    const task = this.taskManagerService.createTask(
      33,
      id,
      true,
      lock ? 'userManager.task.lockUser' : 'userManager.task.unlockUser',
      'pi pi-lock',
      `usermod -${lock ? 'L' : 'U'} '${username}'`,
    );
    if (this.taskManagerService.hasTask(task)) {
      this.taskManagerService.removeTask(task);
    } else {
      this.taskManagerService.scheduleTask(task);
    }
  }

  /**
   * Queue a password change for a user. The password is applied non-interactively
   * in the terminal via `chpasswd` once the pending operations are executed.
   * @param username The username.
   * @param password The new plaintext password.
   */
  queuePassword(username: string, password: string): void {
    const id = `user-manager-password-${username}`;
    const task = this.taskManagerService.createTask(
      34,
      id,
      true,
      'userManager.task.password',
      'pi pi-key',
      `printf '%s\\n' ${this.shellQuote(`${username}:${password}`)} | chpasswd`,
    );
    const existing = this.taskManagerService.findTaskById(id);
    if (existing) {
      this.taskManagerService.removeTask(existing);
    }
    this.taskManagerService.scheduleTask(task);
  }

  /**
   * Quote a string for safe use inside single quotes in a shell command.
   * @param value The value to quote.
   */
  private shellQuote(value: string): string {
    return `'${value.replaceAll("'", `'\\''`)}'`;
  }

  /**
   * Check whether a lock or unlock task is currently queued for a user.
   * @param username The username.
   * @returns 'lock' | 'unlock' | null depending on what is queued.
   */
  pendingLock(username: string): 'lock' | 'unlock' | null {
    if (this.taskManagerService.findTaskById(`user-manager-lock-${username}`)) return 'lock';
    if (this.taskManagerService.findTaskById(`user-manager-unlock-${username}`)) return 'unlock';
    return null;
  }

  /**
   * Check whether a password task is currently queued for a user.
   * @param username The username.
   */
  pendingPassword(username: string): boolean {
    return this.taskManagerService.findTaskById(`user-manager-password-${username}`) !== null;
  }

  /**
   * Clear any pending changes for a user.
   * @param username The username.
   */
  clearUserChanges(username: string): void {
    this.wantedUsers.update((wanted) => {
      const next = new Map(wanted);
      next.delete(username);
      return next;
    });
    this.wantedMemberships.update((memberships) => {
      const next = new Map(memberships);
      next.delete(username);
      return next;
    });
  }

  /**
   * Clear any pending change for a group.
   * @param group The group name.
   */
  clearGroupChanges(group: string): void {
    this.wantedGroups.update((wanted) => {
      const next = new Map(wanted);
      next.delete(group);
      return next;
    });
  }
}
