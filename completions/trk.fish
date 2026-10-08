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
complete -c trk -n "__fish_trk_needs_command" -f -a "add" -d 'Add X: a new task under the current task; you descend'
complete -c trk -n "__fish_trk_needs_command" -f -a "by" -d 'Goal by X: a direct task under the active goal'
complete -c trk -n "__fish_trk_needs_command" -f -a "then" -d 'Once this is done, then X: make X the parent of a picked task'
complete -c trk -n "__fish_trk_needs_command" -f -a "and" -d 'And X: another task at the same level as the target'
complete -c trk -n "__fish_trk_needs_command" -f -a "done" -d 'Finish the current task and walk back up'
complete -c trk -n "__fish_trk_needs_command" -f -a "drop" -d 'Abandon the current task'
complete -c trk -n "__fish_trk_needs_command" -f -a "stop" -d 'Leave the current task unfinished, with a reason'
complete -c trk -n "__fish_trk_needs_command" -f -a "go" -d 'Resume the current task; show how I got here'
complete -c trk -n "__fish_trk_needs_command" -f -a "why" -d 'Chain of notes from the root to the current task'
complete -c trk -n "__fish_trk_needs_command" -f -a "note" -d 'Read or add note lines on a task'
complete -c trk -n "__fish_trk_needs_command" -f -a "rename" -d 'Change a task\'s text'
complete -c trk -n "__fish_trk_needs_command" -f -a "switch" -d 'Switch to another task'
complete -c trk -n "__fish_trk_needs_command" -f -a "list" -d 'The whole task tree of the active goal'
complete -c trk -n "__fish_trk_needs_command" -f -a "undo" -d 'Undo the last N changes'
complete -c trk -n "__fish_trk_needs_command" -f -a "goal" -d 'Goals: show, list, new, switch, done, rename, reopen'
complete -c trk -n "__fish_trk_needs_command" -f -a "jot" -d 'Quick capture to the inbox'
complete -c trk -n "__fish_trk_needs_command" -f -a "inbox" -d 'View the inbox, take or drop items'
complete -c trk -n "__fish_trk_needs_command" -f -a "log" -d 'What got finished, by day'
complete -c trk -n "__fish_trk_needs_command" -f -a "at" -d 'Schedule a task'
complete -c trk -n "__fish_trk_needs_command" -f -a "agenda" -d 'Scheduled tasks across all goals'
complete -c trk -n "__fish_trk_needs_command" -f -a "start" -d 'Live task list in a terminal pane'
complete -c trk -n "__fish_trk_needs_command" -f -a "config" -d 'Where the task list lives'
complete -c trk -n "__fish_trk_needs_command" -f -a "prompt" -d 'One short line for a shell prompt'
complete -c trk -n "__fish_trk_needs_command" -f -a "completions" -d 'Print shell completion script'
complete -c trk -n "__fish_trk_using_subcommand add" -s w -l why -d 'Supply the why inline (skips the prompt)' -r
complete -c trk -n "__fish_trk_using_subcommand add" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand add" -s s -s p -l switch -d 'Switch to choose the parent task instead of using the current task'
complete -c trk -n "__fish_trk_using_subcommand add" -s n -l stay -d 'Do not move the cursor onto the new task'
complete -c trk -n "__fish_trk_using_subcommand add" -l no-why -d 'Do not ask for a why'
complete -c trk -n "__fish_trk_using_subcommand add" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand by" -s w -l why -r
complete -c trk -n "__fish_trk_using_subcommand by" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand by" -l no-why
complete -c trk -n "__fish_trk_using_subcommand by" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand then" -s w -l why -r
complete -c trk -n "__fish_trk_using_subcommand then" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand then" -s c -l current -d 'Use the current task instead of picking one'
complete -c trk -n "__fish_trk_using_subcommand then" -l no-why
complete -c trk -n "__fish_trk_using_subcommand then" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand and" -s w -l why -r
complete -c trk -n "__fish_trk_using_subcommand and" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand and" -s s -s p -l switch
complete -c trk -n "__fish_trk_using_subcommand and" -l no-why
complete -c trk -n "__fish_trk_using_subcommand and" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand done" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand done" -s f -l force -d 'Finish even if the task has open sub-tasks'
complete -c trk -n "__fish_trk_using_subcommand done" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand drop" -s w -l why -r
complete -c trk -n "__fish_trk_using_subcommand drop" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand drop" -s f -l force
complete -c trk -n "__fish_trk_using_subcommand drop" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand stop" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand stop" -s s -s p -l switch
complete -c trk -n "__fish_trk_using_subcommand stop" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand go" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand go" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand why" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand why" -s s -s p -l switch
complete -c trk -n "__fish_trk_using_subcommand why" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand note" -s r -l replace -r
complete -c trk -n "__fish_trk_using_subcommand note" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand note" -s c -l clear
complete -c trk -n "__fish_trk_using_subcommand note" -s s -s p -l switch
complete -c trk -n "__fish_trk_using_subcommand note" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand rename" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand rename" -s s -s p -l switch
complete -c trk -n "__fish_trk_using_subcommand rename" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand switch" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand switch" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand list" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand list" -s a -l all
complete -c trk -n "__fish_trk_using_subcommand list" -l all-goals
complete -c trk -n "__fish_trk_using_subcommand list" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand undo" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand undo" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -f -a "show" -d 'Print title, id, active marker, and counts'
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -f -a "list" -d 'Numbered list of goals'
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -f -a "new" -d 'Create an open goal and make it active'
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -f -a "switch" -d 'Make another open goal active'
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -f -a "done" -d 'Mark the active goal done'
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -f -a "rename" -d 'Rename the active goal'
complete -c trk -n "__fish_trk_using_subcommand goal; and not __fish_seen_subcommand_from show list new switch done rename reopen" -f -a "reopen" -d 'Reopen a done goal and make it active'
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from show" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from list" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from new" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from new" -s n -l stay
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from new" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from switch" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from switch" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from done" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from done" -s f -l force
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from done" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from rename" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from rename" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from reopen" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand goal; and __fish_seen_subcommand_from reopen" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand jot" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand jot" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand inbox; and not __fish_seen_subcommand_from take drop" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand inbox; and not __fish_seen_subcommand_from take drop" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand inbox; and not __fish_seen_subcommand_from take drop" -f -a "take" -d 'Take an item out of the inbox into a goal'
complete -c trk -n "__fish_trk_using_subcommand inbox; and not __fish_seen_subcommand_from take drop" -f -a "drop" -d 'Drop an item from the inbox'
complete -c trk -n "__fish_trk_using_subcommand inbox; and __fish_seen_subcommand_from take" -l goal -r
complete -c trk -n "__fish_trk_using_subcommand inbox; and __fish_seen_subcommand_from take" -l new-goal -r
complete -c trk -n "__fish_trk_using_subcommand inbox; and __fish_seen_subcommand_from take" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand inbox; and __fish_seen_subcommand_from take" -s s -s p -l switch -d 'Switch to choose the parent task instead of making a root'
complete -c trk -n "__fish_trk_using_subcommand inbox; and __fish_seen_subcommand_from take" -s a -l activate -d 'Make the chosen goal active'
complete -c trk -n "__fish_trk_using_subcommand inbox; and __fish_seen_subcommand_from take" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand inbox; and __fish_seen_subcommand_from drop" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand inbox; and __fish_seen_subcommand_from drop" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand log" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand log" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand at" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand at" -s s -s p -l switch
complete -c trk -n "__fish_trk_using_subcommand at" -l clear -d 'Remove the schedule'
complete -c trk -n "__fish_trk_using_subcommand at" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand agenda" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand agenda" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand start" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand start" -l focus -d 'Start in focus view'
complete -c trk -n "__fish_trk_using_subcommand start" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand config; and not __fish_seen_subcommand_from show path" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand config; and not __fish_seen_subcommand_from show path" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand config; and not __fish_seen_subcommand_from show path" -f -a "show" -d 'Print effective values'
complete -c trk -n "__fish_trk_using_subcommand config; and not __fish_seen_subcommand_from show path" -f -a "path" -d 'Print only the config file path'
complete -c trk -n "__fish_trk_using_subcommand config; and __fish_seen_subcommand_from show" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand config; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand config; and __fish_seen_subcommand_from path" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand config; and __fish_seen_subcommand_from path" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand prompt" -l max -r
complete -c trk -n "__fish_trk_using_subcommand prompt" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand prompt" -s h -l help -d 'Print help'
complete -c trk -n "__fish_trk_using_subcommand completions" -l color -d 'When to colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c trk -n "__fish_trk_using_subcommand completions" -s h -l help -d 'Print help'
