# The App Catalog page (Discover). App names and descriptions come from
# the catalog in English.

discover-search = Search apps
discover-search-hint = Search apps…
discover-sort = Sort apps
discover-all = All
discover-sort-name = Name
discover-sort-selected = Selected first
discover-actions = More for { $name }
discover-unreadable = The installed apps could not be read, so no row shows what is installed. Installing an installed app changes nothing.
discover-problem = Some categories could not be loaded. The others are shown.
discover-nothing-matches-detail = The search looks at names, descriptions and package names in every category.
# A chip picked a category without a match while others have some. $category is the category's name, $search what the user typed.
discover-nothing-in-category = No app in { $category } matches “{ $search }”
discover-nothing-in-category-detail = Other categories have apps that match.
# $count is how many apps of every category match the search.
discover-show-all-matches =
    { $count ->
        [one] Show the Match
       *[other] Show All { $count } Matches
    }
discover-none = No apps to show
discover-none-detail = Apps that the configured repositories do not offer are hidden.

# The categories, by their catalog id.
discover-category-documents = Documents
discover-category-games = Games
discover-category-internet = Internet
discover-category-multimedia = Multimedia
discover-category-other = Other
discover-category-science = Science
discover-category-security = Security
discover-category-utilities = Utilities
discover-problem-reason = Some categories could not be loaded. The others are shown. { $reason }

discover-back = Back
discover-back-name = Back to App Catalog
discover-details-packages = Packages
discover-details-website = Website
discover-package-from = { $package } from { $repository }
