
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
            cand add 'Add a task under the goal'
            cand then 'Add a task as a parent of the most recently added task'
            cand and 'Add a task as a sibling of the most recently added task'
            cand before 'Add a task as a sibling before the most recently added task'
            cand done 'Complete the current task'
            cand time 'Start timing the current task'
            cand stop 'Stop timing the current task'
            cand drop 'Delete a task, using the picker'
            cand switch 'Switch tasks using the picker'
            cand list 'Show the current task, its siblings, and its parents'
            cand view 'Browse the whole task list, assigning dates to tasks'
            cand undo 'Undo the last N changes'
            cand new 'Create a goal'
            cand progress 'Show the whole goal as a tree with progress'
            cand delete 'Delete a goal and every task under it'
            cand agenda 'Tasks by the date they are assigned, org-agenda style'
            cand jot 'Capture a task in the inbox'
            cand inbox 'Open the inbox in the picker'
            cand log 'What got finished, by day'
            cand completions 'Install shell completions for the detected shell'
        }
        &'trk;add'= {
            cand -w 'Explain why this task exists (skips the prompt)'
            cand --why 'Explain why this task exists (skips the prompt)'
            cand --color 'When to colorize output'
            cand --no-why 'Do not ask for a why'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;then'= {
            cand -w 'Explain why this task exists (skips the prompt)'
            cand --why 'Explain why this task exists (skips the prompt)'
            cand --color 'When to colorize output'
            cand -n 'Wrap the current task instead of the most recently added one'
            cand --next 'Wrap the current task instead of the most recently added one'
            cand --no-why 'Do not ask for a why'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;and'= {
            cand -w 'Explain why this task exists (skips the prompt)'
            cand --why 'Explain why this task exists (skips the prompt)'
            cand --color 'When to colorize output'
            cand --no-why 'Do not ask for a why'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;before'= {
            cand -w 'Explain why this task exists (skips the prompt)'
            cand --why 'Explain why this task exists (skips the prompt)'
            cand --color 'When to colorize output'
            cand --no-why 'Do not ask for a why'
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
        &'trk;time'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;stop'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;drop'= {
            cand --color 'When to colorize output'
            cand -y 'Skip the confirmation prompt when the task has sub-tasks'
            cand --yes 'Skip the confirmation prompt when the task has sub-tasks'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;switch'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand goal 'Switch goals using the picker'
        }
        &'trk;switch;goal'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;list'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand goal 'List all goals'
        }
        &'trk;list;goal'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;view'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;undo'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;new'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand goal 'Create a new goal and make it active'
        }
        &'trk;new;goal'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;progress'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand goal 'All tasks in this goal as a single tree'
        }
        &'trk;progress;goal'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;delete'= {
            cand --color 'When to colorize output'
            cand -h 'Print help'
            cand --help 'Print help'
            cand goal 'Delete a goal and all of its tasks'
        }
        &'trk;delete;goal'= {
            cand --color 'When to colorize output'
            cand -y 'Skip the confirmation prompt'
            cand --yes 'Skip the confirmation prompt'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'trk;agenda'= {
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
        }
        &'trk;log'= {
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
