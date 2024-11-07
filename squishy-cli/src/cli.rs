use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    help_template = "{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}",
    arg_required_else_help = true
)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// AppImage specific tasks
    #[command(arg_required_else_help = true)]
    #[clap(name = "appimage", alias = "ai")]
    AppImage {
        /// Path to appimage file
        #[arg(required = true)]
        file: PathBuf,

        /// Name of the app
        #[arg(required = true, long)]
        app: String,

        /// Whether to search for icon
        #[arg(required = false, long)]
        icon: bool,

        /// Whether to search for desktop file
        #[arg(required = false, long)]
        desktop: bool,

        /// Whether to search for appstream file
        #[arg(required = false, long)]
        appstream: bool,

        /// Whether to write files to disk
        #[arg(required = false, long, short)]
        write: Option<Option<PathBuf>>,
    },
}
