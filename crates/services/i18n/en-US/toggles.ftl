# The catalog toggle pages (Components, …).

toggles-search = Search this page
# The chip that shows every section.
toggles-all = All
toggles-search-hint = Search this page…
toggles-nothing-matches = Nothing matches the search
# $search is what the user typed.
toggles-nothing-matches-detail = No entry of this page has “{ $search }” in its name or description.
toggles-clear-search = Clear search
# A chip picked a section without a match while other sections have some. $section is the section's title, $search what the user typed.
toggles-nothing-in-section = Nothing in { $section } matches
toggles-nothing-in-section-detail = Other sections have entries with “{ $search }” in their name or description.
# $count is how many entries of the whole page match the search.
toggles-show-all-matches =
    { $count ->
        [one] Show the match
       *[other] Show all { $count } matches
    }
# $on and $total are counts of the entries of a section, a chip or the page, shown as "2/8".
toggles-section-count = { $on }/{ $total }
# The count while some states could not be read, so how many are on is not known. $total is the count of entries.
toggles-section-count-unread = ?/{ $total }
toggles-unknown =
    { $count ->
        [one] The state of one entry could not be read, so its switch may show the wrong state.
       *[other] The state of { $count } entries could not be read, so their switches may show the wrong state.
    }
toggles-state-unknown = State not known.
# $names lists the titles of the entries, joined by commas.
toggles-needs = Needs { $names }
toggles-needs-any = Needs one of: { $names }
toggles-turns-on = Turns on with Apply
toggles-enable = Enable
toggles-disable = Disable
toggles-turns-off = Turns off with Apply
toggles-dismiss = Dismiss

# Why the catalog could not be loaded.
toggles-problem = The list could not be loaded.
toggles-problem-unreadable = A catalog file could not be read.
toggles-problem-syntax = A catalog file is damaged.
toggles-problem-invalid = An entry of a catalog file breaks a rule.
