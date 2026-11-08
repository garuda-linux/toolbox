//! The Settings page's data: what it changes and where that is kept.
//!
//! Theme and reduced motion are kept with the window state, the colour
//! schemes in the file, so the command line reaches them. The file and the
//! autostart link are written on a thread of their own, so a slow disk
//! never holds a frame.

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::mpsc::{Sender, channel};

use egui::Context;
use serde::{Deserialize, Serialize};
use toolbox_catalog::SchemeMode;
use toolbox_config::{Autostart, LogLevel, RefreshInterval, Settings};
use toolbox_gui_kit::toasts::{ToastKind, raise};
use toolbox_plan::SchemeId;

use crate::command::{AppCommand, Command};
use crate::i18n;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeChoice {
    #[default]
    FollowDesktop,
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionChoice {
    #[default]
    FollowSystem,
    Reduced,
    Full,
}

/// A change on the Settings page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsCommand {
    Language(Option<String>),
    Theme(ThemeChoice),
    /// The scheme of one mode, `None` takes the desktop's colours.
    Scheme(SchemeMode, Option<SchemeId>),
    Motion(MotionChoice),
    StartAtLogin(bool),
    Refresh(RefreshInterval),
    UserUnits(bool),
    CopyReports(bool),
    Aur(bool),
    LogLevel(LogLevel),
}

impl From<SettingsCommand> for Command {
    fn from(command: SettingsCommand) -> Self {
        Self::App(AppCommand::Setting(command))
    }
}

pub trait SettingsStore: Send + Sync + std::fmt::Debug {
    fn save(&self, settings: Settings);
    fn set_autostart(&self, on: bool);
}

/// `settings.toml` and the autostart link, written by one thread in the
/// order the changes came.
#[derive(Debug)]
pub struct FileStore {
    jobs: Sender<Job>,
}

#[derive(Debug)]
enum Job {
    Save(Box<Settings>),
    Autostart(bool),
}

impl FileStore {
    /// A store without a window to tell of a failure, only the log.
    #[cfg(test)]
    pub fn new(path: PathBuf, autostart: Autostart) -> Self {
        Self::spawn(path, autostart, None)
    }

    /// A failure to save raises a note in `ctx`'s window.
    pub fn with_window(path: PathBuf, autostart: Autostart, ctx: Context) -> Self {
        Self::spawn(path, autostart, Some(ctx))
    }

    fn spawn(path: PathBuf, autostart: Autostart, ctx: Option<Context>) -> Self {
        let (jobs, queue) = channel::<Job>();
        let worker = std::thread::Builder::new()
            .name("settings".to_owned())
            .spawn(move || {
                for job in queue {
                    let failed = match job {
                        Job::Save(settings) => settings
                            .save(&path)
                            .map_err(|error| error.to_string())
                            .err(),
                        Job::Autostart(on) => {
                            let done = if on {
                                autostart.enable()
                            } else {
                                autostart.disable()
                            };

                            done.map_err(|error| error.to_string()).err()
                        }
                    };

                    if let Some(error) = failed {
                        tracing::warn!("a setting was not saved: {error}");
                        if let Some(ctx) = &ctx {
                            raise(ctx, ToastKind::Error, i18n::text(ctx, "settings-not-saved"));
                            ctx.request_repaint();
                        }
                    }
                }
            });

        if let Err(error) = worker {
            tracing::warn!("no thread to save the settings: {error}");
        }

        Self { jobs }
    }

    fn send(&self, job: Job) {
        if self.jobs.send(job).is_err() {
            tracing::warn!("a setting was not saved: the settings thread is gone");
        }
    }
}

impl SettingsStore for FileStore {
    fn save(&self, settings: Settings) {
        self.send(Job::Save(Box::new(settings)));
    }

    fn set_autostart(&self, on: bool) {
        self.send(Job::Autostart(on));
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SettingsView<'a> {
    pub settings: &'a Settings,
    pub autostart: bool,
    /// The chosen language's code, `None` while it follows the system.
    pub language: Option<&'a str>,
    pub theme: ThemeChoice,
    pub motion: MotionChoice,
}
