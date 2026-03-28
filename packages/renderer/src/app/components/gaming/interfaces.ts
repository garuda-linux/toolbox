export interface StatefulPackage {
  pkgname: string[];
  selected?: boolean;
  disabled?: boolean;
}

export interface Package {
  description?: string;
  icon?: string;
  name: string;
  url?: string;
}

export type FullPackageDefinition = StatefulPackage & Package;

export interface PackageSection {
  name: string;
  hint?: string;
  sections: FullPackageDefinition[];
}
export type PackageSections = PackageSection[];
