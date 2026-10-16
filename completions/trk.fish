# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_trk_global_optspecs
    string join \n color= h/help V/version
end

function __fish_trk_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_trk_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_trk_using_subcommand
    set -l cmd (__fish_trk_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c trk -n "__fish_trk_needs_command" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_needs_command" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_needs_command" -s V -l version -d 'Print version'
complete -c trk -n "__fish_trk_needs_command" -f -a "add" -d 'Add a task under the goal'
complete -c trk -n "__fish_trk_needs_command" -f -a "then" -d 'Add a task as a parent of the most recently added task'
complete -c trk -n "__fish_trk_needs_command" -f -a "and" -d 'Add a task as a sibling of the most recently added task'
complete -c trk -n "__fish_trk_needs_command" -f -a "before" -d 'Add a task as a sibling before the most recently added task'
complete -c trk -n "__fish_trk_needs_command" -f -a "done" -d 'Complete the current task'
complete -c trk -n "__fish_trk_needs_command" -f -a "time" -d 'Start timing the current task'
complete -c trk -n "__fish_trk_needs_command" -f -a "stop" -d 'Stop timing the current task'
complete -c trk -n "__fish_trk_needs_command" -f -a "drop" -d 'Delete a task, using the picker'
complete -c trk -n "__fish_trk_needs_command" -f -a "switch" -d 'Switch tasks using the picker'
complete -c trk -n "__fish_trk_needs_command" -f -a "list" -d 'Show the current task, its siblings, and its parents'
complete -c trk -n "__fish_trk_needs_command" -f -a "undo" -d 'Undo the last N changes'
complete -c trk -n "__fish_trk_needs_command" -f -a "new" -d 'Create a goal'
complete -c trk -n "__fish_trk_needs_command" -f -a "progress" -d 'Show the whole goal as a tree with progress'
complete -c trk -n "__fish_trk_needs_command" -f -a "delete" -d 'Delete a goal and every task under it'
complete -c trk -n "__fish_trk_needs_command" -f -a "jot" -d 'Capture a task in the inbox'
complete -c trk -n "__fish_trk_needs_command" -f -a "inbox" -d 'Open the inbox in the picker'
complete -c trk -n "__fish_trk_needs_command" -f -a "log" -d 'What got finished, by day'
complete -c trk -n "__fish_trk_needs_command" -f -a "completions" -d 'Install shell completions for the detected shell'
complete -c trk -n "__fish_trk_using_subcommand add" -s w -l why -d 'Explain why this task exists (skips the prompt)' -r
complete -c trk -n "__fish_trk_using_subcommand add" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand add" -l no-why -d 'Do not ask for a why'
complete -c trk -n "__fish_trk_using_subcommand add" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand then" -s w -l why -d 'Explain why this task exists (skips the prompt)' -r
complete -c trk -n "__fish_trk_using_subcommand then" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand then" -s n -l next -d 'Wrap the current task instead of the most recently added one'
complete -c trk -n "__fish_trk_using_subcommand then" -l no-why -d 'Do not ask for a why'
complete -c trk -n "__fish_trk_using_subcommand then" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand and" -s w -l why -d 'Explain why this task exists (skips the prompt)' -r
complete -c trk -n "__fish_trk_using_subcommand and" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand and" -l no-why -d 'Do not ask for a why'
complete -c trk -n "__fish_trk_using_subcommand and" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand before" -s w -l why -d 'Explain why this task exists (skips the prompt)' -r
complete -c trk -n "__fish_trk_using_subcommand before" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand before" -l no-why -d 'Do not ask for a why'
complete -c trk -n "__fish_trk_using_subcommand before" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand done" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand done" -s f -l force -d 'Finish even if the task has open sub-tasks'
complete -c trk -n "__fish_trk_using_subcommand done" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand time" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand time" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand stop" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand stop" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand drop" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand drop" -s y -l yes -d 'Skip the confirmation prompt when the task has sub-tasks'
complete -c trk -n "__fish_trk_using_subcommand drop" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand switch; and not __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand switch; and not __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand switch; and not __fish_seen_subcommand_from goal" -f -a "goal" -d 'Switch goals using the picker'
complete -c trk -n "__fish_trk_using_subcommand switch; and __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand switch; and __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand list; and not __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand list; and not __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand list; and not __fish_seen_subcommand_from goal" -f -a "goal" -d 'List all goals'
complete -c trk -n "__fish_trk_using_subcommand list; and __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand list; and __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand undo" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand undo" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand new; and not __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand new; and not __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand new; and not __fish_seen_subcommand_from goal" -f -a "goal" -d 'Create a new goal and make it active'
complete -c trk -n "__fish_trk_using_subcommand new; and __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand new; and __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand progress; and not __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand progress; and not __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand progress; and not __fish_seen_subcommand_from goal" -f -a "goal" -d 'All tasks in this goal as a single tree'
complete -c trk -n "__fish_trk_using_subcommand progress; and __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand progress; and __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand delete; and not __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand delete; and not __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand delete; and not __fish_seen_subcommand_from goal" -f -a "goal" -d 'Delete a goal and all of its tasks'
complete -c trk -n "__fish_trk_using_subcommand delete; and __fish_seen_subcommand_from goal" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand delete; and __fish_seen_subcommand_from goal" -s y -l yes -d 'Skip the confirmation prompt'
complete -c trk -n "__fish_trk_using_subcommand delete; and __fish_seen_subcommand_from goal" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand jot" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand jot" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand inbox" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand inbox" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand log" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand log" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand completions" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand completions" -s h -l help -d 'Print help'
