# Command palette and shortcuts

The command palette finds pages, commands, Maintenance operations, the
apps of **App Catalog** and the packages of the repositories and the AUR,
so you can reach them with the keyboard alone.

![The open command palette listing pages and commands](images/palette-open.png)

## Find something with the command palette

1. Press `Ctrl+k`, or click **Search** at the top of the sidebar, and type.
2. Move to the entry you want with the `Up` and `Down` keys and press
   `Enter`, or click it.

Press `Esc` to close the palette without choosing.

With no text, the palette lists the pages and commands. Once you type, an
entry shows when its name holds the text, an app or a package also when its
description does. Upper and lower case do not matter. Each group shows its
best five entries: an exact name first, then names that start with the text,
then names that hold it, then entries found by their description. The group
with the best entry comes first. A group with more ends with **Show All**: for
apps and packages it opens their page searched for the text, for the others it
shows the whole group in the palette. When nothing fits, the list stays empty.

Packages are read when the palette opens, so the first look may take a
moment. AUR packages show only while the AUR is turned on in **Settings**.

### What an entry does

| Entry                   | Shown at the right                           | Choosing it                                               |
| ----------------------- | -------------------------------------------- | --------------------------------------------------------- |
| A page, such as Kernels | its shortcut, if any                         | Opens the page                                            |
| A command               | its shortcut, if any                         | Runs it, see below                                        |
| A Maintenance operation | **Maintenance** and tab                      | Opens **Maintenance** on the tab that holds the operation |
| An app, such as Firefox | **App** and its category                     | Opens **App Catalog** with the app on top                 |
| A package               | its repository, and **Installed** when it is | Opens the package on **Packages**                         |
| An AUR package          | **AUR**, and **Installed** when it is        | Opens the package on **Packages**                         |

The commands are **Apply Changes**, **Review Changes** (only while changes
wait), **Remove All Changes**, **Search This Page**, **Help for This Page**,
**Toggle Sidebar**, **Toggle Console**, **Zoom In**, **Zoom Out**, **Reset
Zoom** and **Quit**. They run at once, as their shortcut would. **Remove All
Changes** asks first. Pages, operations, apps and packages only open. They never
change anything by themselves.

### Keys in the palette

| Key          | Does                                  |
| ------------ | ------------------------------------- |
| `Up`, `Down` | Moves one entry, wrapping at the ends |
| `Enter`      | Takes the marked entry                |
| `Esc`        | Closes the palette                    |

## Keyboard shortcuts

These keys work on every page.

| Shortcut             | Does                                                  |
| -------------------- | ----------------------------------------------------- |
| `Ctrl+k`             | Opens the command palette                             |
| `Ctrl+Enter`         | Applies the queued changes                            |
| `Ctrl+b`             | Shows or hides the page names in the sidebar          |
| `Ctrl+j`             | Shows or hides the **Console**                        |
| `Ctrl+f`             | Puts the cursor into the search of the open page      |
| `Ctrl+1` to `Ctrl+9` | Opens the first nine pages of the sidebar             |
| `F5`                 | Reads the open page again, as its **Read Again** does |
| `F1`                 | Opens the [User guide](guide.md) on the open page     |
| `Ctrl++`, `Ctrl+=`   | Shows everything in the window larger, up to 150 %    |
| `Ctrl+-`             | Shows everything in the window smaller, down to 80 %  |
| `Ctrl+0`             | Shows everything at its normal size again             |
| `Ctrl+q`             | Quits the app. Queued changes are kept                |
| `Ctrl+Tab`           | Leaves the terminal in the **Console**                |

`F5` does nothing on a page without **Read Again**. The zoom is kept for
the next start.

> **Note:** While the terminal of the **Console** has the keyboard focus,
> keys go to the running program instead. An editor that runs there needs
> them. Press `Ctrl+Tab` first to leave the terminal.

The **Setup Assistant** has two more: `Alt+Right` goes to the next step and
`Alt+Left` goes back.

## Use the global menu on Plasma

On Plasma, the window also offers its commands as a global menu. It shows
where Plasma shows application menus: behind the application menu button
of the title bar, or in a **Global Menu** widget on a panel. Other desktops
show no global menu, and the window works the same without it.

| Menu     | Entries                                                                      |
| -------- | ---------------------------------------------------------------------------- |
| **File** | **Review and Apply…**, **Remove All Changes…**, **Quit**                     |
| **View** | **Expanded Sidebar**, **Console**, **Zoom In**, **Zoom Out**, **Reset Zoom** |
| **Go**   | **Search This Page**, **Command Palette**, then the pages of the sidebar     |
| **Help** | **Help for This Page**                                                       |

**Review and Apply…** opens the list of queued changes. Click **Apply**
there to apply them. No menu entry applies changes by itself, because any
program of your session can choose a menu entry. **Review and Apply…** and
**Remove All Changes…** are dimmed while no change waits. **Go** marks the
open page, **View** whether the sidebar shows the page names and whether
the **Console** is open. Entries show their shortcut.

The global menu belongs to the window. The command line has no
counterpart.

## See also

- [The Console](console.md)
- [Maintenance](maintenance.md)
- [App Catalog](discover.md)
- [Settings and About](settings.md)
