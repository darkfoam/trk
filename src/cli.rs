use clap::{Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorArg {
    Auto,
    Always,
    Never,
}

impl ColorArg {
    pub fn to_toggle(self) -> crate::config::Toggle {
        match self {
            ColorArg::Auto => crate::config::Toggle::Auto,
            ColorArg::Always => crate::config::Toggle::Always,
            ColorArg::Never => crate::config::Toggle::Never,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Sh,
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

impl Shell {
    pub fn name(self) -> &'static str {
        match self {
            Shell::Sh => "sh",
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Powershell => "powershell",
            Shell::Elvish => "elvish",
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "trk",
    version,
    about = "adhd compliant task tracking",
    max_term_width = 80,
    subcommand_required = false,
    arg_required_else_help = false,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// When to colorize output.
    #[arg(long, global = true, value_enum)]
    pub color: Option<ColorArg>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Add a task under the goal.
    Add(AddArgs),
    /// Add a task as a parent of the most recently added task.
    Then(ThenArgs),
    /// Add a task as a sibling of the most recently added task.
    And(AndArgs),
    /// Add a task as a sibling before the most recently added task.
    Before(BeforeArgs),
    /// Complete the current task.
    #[command(alias = "d")]
    Done(DoneArgs),
    /// Start timing the current task.
    Time,
    /// Stop timing the current task.
    Stop,
    /// Delete a task, using the picker.
    Drop(DropArgs),
    /// Switch tasks using the picker.
    Switch(SwitchArgs),
    /// Show the current task, its siblings, and its parents.
    #[command(alias = "l")]
    List(ListArgs),
    /// Browse the whole task list, assigning dates to tasks.
    View,
    /// Undo the last N changes.
    #[command(alias = "u")]
    Undo(UndoArgs),
    /// Create a goal.
    New(NewArgs),
    /// Show the whole goal as a tree with progress.
    Progress(ProgressArgs),
    /// Delete a goal and every task under it.
    Delete(DeleteArgs),
    /// Tasks by the date they are assigned, org-agenda style.
    Agenda,
    /// Capture a task in the inbox.
    #[command(alias = "i")]
    Jot(JotArgs),
    /// Open the inbox in the picker.
    Inbox,
    /// What got finished, by day.
    Log(LogArgs),
    /// Install shell completions for the detected shell.
    Completions(CompletionsArgs),
}

#[derive(clap::Args, Debug)]
pub struct AddArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    /// Explain why this task exists (skips the prompt).
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
    /// Do not ask for a why.
    #[arg(long)]
    pub no_why: bool,
}

#[derive(clap::Args, Debug)]
pub struct ThenArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    /// Wrap the current task instead of the most recently added one.
    #[arg(short = 'n', long)]
    pub next: bool,
    /// Explain why this task exists (skips the prompt).
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
    /// Do not ask for a why.
    #[arg(long)]
    pub no_why: bool,
}

#[derive(clap::Args, Debug)]
pub struct AndArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    /// Explain why this task exists (skips the prompt).
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
    /// Do not ask for a why.
    #[arg(long)]
    pub no_why: bool,
}

#[derive(clap::Args, Debug)]
pub struct BeforeArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    /// Explain why this task exists (skips the prompt).
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
    /// Do not ask for a why.
    #[arg(long)]
    pub no_why: bool,
}

#[derive(clap::Args, Debug)]
pub struct DoneArgs {
    /// Finish even if the task has open sub-tasks.
    #[arg(short = 'f', long)]
    pub force: bool,
}

#[derive(clap::Args, Debug)]
pub struct DropArgs {
    /// Skip the confirmation prompt when the task has sub-tasks.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(clap::Args, Debug)]
pub struct SwitchArgs {
    #[command(subcommand)]
    pub what: Option<SwitchWhat>,
}

#[derive(Subcommand, Debug)]
pub enum SwitchWhat {
    /// Switch goals using the picker.
    Goal,
}

#[derive(clap::Args, Debug)]
pub struct ListArgs {
    #[command(subcommand)]
    pub what: Option<ListWhat>,
}

#[derive(Subcommand, Debug)]
pub enum ListWhat {
    /// List all goals.
    Goal,
}

#[derive(clap::Args, Debug)]
pub struct UndoArgs {
    #[arg(value_name = "N")]
    pub count: Option<usize>,
}

#[derive(clap::Args, Debug)]
pub struct NewArgs {
    #[command(subcommand)]
    pub what: NewWhat,
}

#[derive(Subcommand, Debug)]
pub enum NewWhat {
    /// Create a new goal and make it active.
    Goal(NewGoalArgs),
}

#[derive(clap::Args, Debug)]
pub struct NewGoalArgs {
    #[arg(value_name = "TITLE")]
    pub title: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct ProgressArgs {
    #[command(subcommand)]
    pub what: ProgressWhat,
}

#[derive(Subcommand, Debug)]
pub enum ProgressWhat {
    /// All tasks in this goal as a single tree.
    Goal,
}

#[derive(clap::Args, Debug)]
pub struct DeleteArgs {
    #[command(subcommand)]
    pub what: DeleteWhat,
}

#[derive(Subcommand, Debug)]
pub enum DeleteWhat {
    /// Delete a goal and all of its tasks.
    Goal(DeleteGoalArgs),
}

#[derive(clap::Args, Debug)]
pub struct DeleteGoalArgs {
    /// Skip the confirmation prompt.
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(clap::Args, Debug)]
pub struct JotArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct LogArgs {
    #[arg(value_name = "WHEN")]
    pub when: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct CompletionsArgs {
    /// Install for this shell instead of auto-detecting.
    #[arg(value_enum)]
    pub shell: Option<Shell>,
}
