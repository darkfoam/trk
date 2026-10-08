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
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
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
    /// Add X: a new task under the current task; you descend.
    Add(AddArgs),
    /// Goal by X: a direct task under the active goal.
    By(ByArgs),
    /// Once this is done, then X: make X the parent of a picked task.
    Then(ThenArgs),
    /// And X: another task at the same level as the target.
    And(AndArgs),
    /// Finish the current task and walk back up.
    #[command(alias = "d")]
    Done(DoneArgs),
    /// Abandon the current task.
    Drop(DropArgs),
    /// Leave the current task unfinished, with a reason.
    #[command(alias = "s")]
    Stop(StopArgs),
    /// Resume the current task; show how I got here.
    Go,
    /// Chain of notes from the root to the current task.
    #[command(alias = "w")]
    Why(WhyArgs),
    /// Read or add note lines on a task.
    #[command(alias = "n")]
    Note(NoteArgs),
    /// Change a task's text.
    Rename(RenameArgs),
    /// Switch to another task.
    Switch(SwitchArgs),
    /// The whole task tree of the active goal.
    #[command(alias = "l")]
    List(ListArgs),
    /// Undo the last N changes.
    #[command(alias = "u")]
    Undo(UndoArgs),
    /// Goals: show, list, new, switch, done, rename, reopen.
    #[command(alias = "g")]
    Goal(GoalArgs),
    /// Quick capture to the inbox.
    #[command(alias = "i")]
    Jot(JotArgs),
    /// View the inbox, take or drop items.
    Inbox(InboxArgs),
    /// What got finished, by day.
    Log(LogArgs),
    /// Schedule a task.
    At(AtArgs),
    /// Scheduled tasks across all goals.
    Agenda,
    /// Live task list in a terminal pane.
    Start(StartArgs),
    /// Where the task list lives.
    Config(ConfigArgs),
    /// One short line for a shell prompt.
    Prompt(PromptArgs),
    /// Print shell completion script.
    Completions(CompletionsArgs),
}

#[derive(clap::Args, Debug)]
pub struct AddArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    /// Switch to choose the parent task instead of using the current task.
    #[arg(short = 's', visible_short_alias = 'p', long)]
    pub switch: bool,
    /// Do not move the cursor onto the new task.
    #[arg(short = 'n', long)]
    pub stay: bool,
    /// Supply the why inline (skips the prompt).
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
    /// Do not ask for a why.
    #[arg(long)]
    pub no_why: bool,
}

#[derive(clap::Args, Debug)]
pub struct ByArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
    #[arg(long)]
    pub no_why: bool,
}

#[derive(clap::Args, Debug)]
pub struct ThenArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    /// Use the current task instead of picking one.
    #[arg(short = 'c', long)]
    pub current: bool,
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
    #[arg(long)]
    pub no_why: bool,
}

#[derive(clap::Args, Debug)]
pub struct AndArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    #[arg(short = 's', visible_short_alias = 'p', long)]
    pub switch: bool,
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
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
    #[arg(short = 'f', long)]
    pub force: bool,
    #[arg(short = 'w', long, value_name = "WHY")]
    pub why: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct StopArgs {
    #[arg(value_name = "REASON")]
    pub reason: Vec<String>,
    #[arg(short = 's', visible_short_alias = 'p', long)]
    pub switch: bool,
}

#[derive(clap::Args, Debug)]
pub struct WhyArgs {
    #[arg(short = 's', visible_short_alias = 'p', long)]
    pub switch: bool,
}

#[derive(clap::Args, Debug)]
pub struct NoteArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    #[arg(short = 'r', long, value_name = "TEXT")]
    pub replace: Option<String>,
    #[arg(short = 'c', long)]
    pub clear: bool,
    #[arg(short = 's', visible_short_alias = 'p', long)]
    pub switch: bool,
}

#[derive(clap::Args, Debug)]
pub struct RenameArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
    #[arg(short = 's', visible_short_alias = 'p', long)]
    pub switch: bool,
}

#[derive(clap::Args, Debug)]
pub struct SwitchArgs {
    #[arg(value_name = "N")]
    pub index: Option<usize>,
}

#[derive(clap::Args, Debug)]
pub struct ListArgs {
    #[arg(short = 'a', long)]
    pub all: bool,
    #[arg(long)]
    pub all_goals: bool,
}

#[derive(clap::Args, Debug)]
pub struct UndoArgs {
    #[arg(value_name = "N")]
    pub count: Option<usize>,
}

#[derive(clap::Args, Debug)]
pub struct GoalArgs {
    #[command(subcommand)]
    pub command: Option<GoalCommand>,
}

#[derive(Subcommand, Debug)]
pub enum GoalCommand {
    /// Print title, id, active marker, and counts.
    Show,
    /// Numbered list of goals.
    List,
    /// Create an open goal and make it active.
    New(GoalNewArgs),
    /// Make another open goal active.
    Switch(GoalSwitchArgs),
    /// Mark the active goal done.
    Done(GoalDoneArgs),
    /// Rename the active goal.
    Rename(GoalRenameArgs),
    /// Reopen a done goal and make it active.
    Reopen(GoalReopenArgs),
}

#[derive(clap::Args, Debug)]
pub struct GoalNewArgs {
    #[arg(value_name = "TITLE")]
    pub title: Vec<String>,
    #[arg(short = 'n', long)]
    pub stay: bool,
}

#[derive(clap::Args, Debug)]
pub struct GoalSwitchArgs {
    #[arg(value_name = "N")]
    pub index: Option<usize>,
}

#[derive(clap::Args, Debug)]
pub struct GoalDoneArgs {
    #[arg(short = 'f', long)]
    pub force: bool,
}

#[derive(clap::Args, Debug)]
pub struct GoalRenameArgs {
    #[arg(value_name = "TITLE")]
    pub title: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct GoalReopenArgs {
    #[arg(value_name = "N")]
    pub index: Option<usize>,
}

#[derive(clap::Args, Debug)]
pub struct JotArgs {
    #[arg(value_name = "TEXT")]
    pub text: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct InboxArgs {
    #[command(subcommand)]
    pub command: Option<InboxCommand>,
}

#[derive(Subcommand, Debug)]
pub enum InboxCommand {
    /// Take an item out of the inbox into a goal.
    Take(InboxTakeArgs),
    /// Drop an item from the inbox.
    Drop(InboxDropArgs),
}

#[derive(clap::Args, Debug)]
pub struct InboxTakeArgs {
    #[arg(value_name = "N")]
    pub index: Option<usize>,
    #[arg(long, value_name = "NAME")]
    pub goal: Option<String>,
    #[arg(long, value_name = "TITLE")]
    pub new_goal: Option<String>,
    /// Switch to choose the parent task instead of making a root.
    #[arg(short = 's', visible_short_alias = 'p', long)]
    pub switch: bool,
    /// Make the chosen goal active.
    #[arg(short = 'a', long)]
    pub activate: bool,
}

#[derive(clap::Args, Debug)]
pub struct InboxDropArgs {
    #[arg(value_name = "N")]
    pub index: Option<usize>,
}

#[derive(clap::Args, Debug)]
pub struct LogArgs {
    #[arg(value_name = "WHEN")]
    pub when: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct AtArgs {
    #[arg(value_name = "WHEN")]
    pub when: Vec<String>,
    #[arg(short = 's', visible_short_alias = 'p', long)]
    pub switch: bool,
    /// Remove the schedule.
    #[arg(long)]
    pub clear: bool,
}

#[derive(clap::Args, Debug)]
pub struct StartArgs {
    /// Start in focus view.
    #[arg(long)]
    pub focus: bool,
}

#[derive(clap::Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: Option<ConfigCommand>,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Print effective values.
    Show,
    /// Print only the config file path.
    Path,
}

#[derive(clap::Args, Debug)]
pub struct PromptArgs {
    #[arg(long, default_value_t = 30)]
    pub max: usize,
}

#[derive(clap::Args, Debug)]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    pub shell: Shell,
}
