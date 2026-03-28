export interface SetupSoftwareItem {
  id: string;
  name: string;
  description: string;
  packages: string[];
  selected?: boolean;
  icon?: string;
  rootCommands?: string[];
  userCommands?: string[];
}

export interface SetupSoftwareCategory {
  id: string;
  name: string;
  items: SetupSoftwareItem[];
}
