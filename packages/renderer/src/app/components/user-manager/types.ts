export interface ManagedUser {
  username: string;
  uid: number;
  gid: number;
  fullName: string;
  home: string;
  shell: string;
  locked: boolean;
  system: boolean;
  groups: string[];
}

export interface ManagedGroup {
  name: string;
  gid: number;
  members: string[];
}

export type UserChangeAction = 'add' | 'remove' | 'modify';

export interface UserChange {
  action: UserChangeAction;
  username: string;
  fullName?: string;
  home?: string;
  shell?: string;
  createHome?: boolean;
  system?: boolean;
  uid?: number;
  password?: string;
}
