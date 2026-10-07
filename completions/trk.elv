
use builtin;
use str;

set edit:completion:arg-completer[trk] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'trk'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'trk'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand need 'What am I doing, and why'
            cand by 'Goal by X: a direct task under the active goal'
            cand then 'Once this is done, then X: wrap the current chain in a new parent'
            cand and 'And X: another task at the same level as the target'
            cand done 'Finish the current task and walk back up'
            cand drop 'Abandon the current task'
            cand stop 'Leave the current task unfinished, with a reason'
            cand go 'Resume the current task; show how I got here'
            cand why 'Chain of notes from the root to the current task'
            cand note 'Read or add note lines on a task'
            cand rename 'Change a task''s text'
            cand switch 'Switch to another task'
            cand list 'The whole task tree of the active goal'
            cand undo 'Undo the last N changes'
            cand goal 'Goals: show, list, new, switch, done, rename, reopen'
            cand jot 'Quick capture to the inbox'
            cand inbox 'View the inbox, take or drop items'
            cand log 'What got finished, by day'
            cand at 'Schedule a task'
            cand agenda 'Scheduled tasks across all goals'
            cand start 'Live task list in a terminal pane'
            cand config 'Where the task list lives'
            cand prompt 'One short line for a shell prompt'
            cand completions 'Print shell completion script'
        }
        &'trk;need'= {
            cand -w 'Supply the why inline (skips the prompt)'
            cand --why 'Supply the why inline (skips the prompt)'
            cand --color 'When to colorize output'
            cand -s 'Switch to choose the target task instead of using the current task'
            cand --switch 'Switch to choose the target task instead of using the current task'
            cand -n 'Do not move the cursor onto the new task'
            cand --stay 'Do not move the cursor onto the new task'
            cand --no-why 'Do not ask for a why'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;by'= {
            cand -w 'w'
            cand --why 'why'
            cand --color 'When to colorize output'
            cand --no-why 'no-why'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;then'= {
            cand -w 'w'
            cand --why 'why'
            cand --color 'When to colorize output'
            cand -s 's'
            cand --switch 'switch'
            cand --no-why 'no-why'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;and'= {
            cand -w 'w'
            cand --why 'why'
            cand --color 'When to colorize output'
            cand -s 's'
            cand --switch 'switch'
            cand --no-why 'no-why'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;done'= {
            cand --color 'When to colorize output'
            cand -f 'Finish even if the task has open sub-tasks'
            cand --force 'Finish even if the task has open sub-tasks'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;drop'= {
            cand -w 'w'
            cand --why 'why'
            cand --color 'When to colorize output'
            cand -f 'f'
            cand --force 'force'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;stop'= {
            cand --color 'When to colorize output'
            cand -s 's'
            cand --switch 'switch'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;go'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;why'= {
            cand --color 'When to colorize output'
            cand -s 's'
            cand --switch 'switch'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;note'= {
            cand -r 'r'
            cand --replace 'replace'
            cand --color 'When to colorize output'
            cand -c 'c'
            cand --clear 'clear'
            cand -s 's'
            cand --switch 'switch'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;rename'= {
            cand --color 'When to colorize output'
            cand -s 's'
            cand --switch 'switch'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;switch'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;list'= {
            cand --color 'When to colorize output'
            cand -a 'a'
            cand --all 'all'
            cand --all-goals 'all-goals'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;undo'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;goal'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand show 'Print title, id, active marker, and counts'
            cand list 'Numbered list of goals'
            cand new 'Create an open goal and make it active'
            cand switch 'Make another open goal active'
            cand done 'Mark the active goal done'
            cand rename 'Rename the active goal'
            cand reopen 'Reopen a done goal and make it active'
        }
        &'trk;goal;show'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;goal;list'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;goal;new'= {
            cand --color 'When to colorize output'
            cand -n 'n'
            cand --stay 'stay'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;goal;switch'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;goal;done'= {
            cand --color 'When to colorize output'
            cand -f 'f'
            cand --force 'force'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;goal;rename'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;goal;reopen'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;jot'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;inbox'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand take 'Take an item out of the inbox into a goal'
            cand drop 'Drop an item from the inbox'
        }
        &'trk;inbox;take'= {
            cand --goal 'goal'
            cand --new-goal 'new-goal'
            cand --color 'When to colorize output'
            cand -s 'Switch to choose the parent task instead of making a root'
            cand --switch 'Switch to choose the parent task instead of making a root'
            cand -a 'Make the chosen goal active'
            cand --activate 'Make the chosen goal active'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;inbox;drop'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;log'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;at'= {
            cand --color 'When to colorize output'
            cand -s 's'
            cand --switch 'switch'
            cand --clear 'Remove the schedule'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;agenda'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;start'= {
            cand --color 'When to colorize output'
            cand --focus 'Start in focus view'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;config'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand show 'Print effective values'
            cand path 'Print only the config file path'
        }
        &'trk;config;show'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;config;path'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;prompt'= {
            cand --max 'max'
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;completions'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
    ]
    $completions[$command]
}
