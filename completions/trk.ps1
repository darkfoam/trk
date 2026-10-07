
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'trk' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'trk'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'trk' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('need', 'need', [CompletionResultType]::ParameterValue, 'What am I doing, and why')
            [CompletionResult]::new('also', 'also', [CompletionResultType]::ParameterValue, 'Also, this other thing (a new top-level task)')
            [CompletionResult]::new('then', 'then', [CompletionResultType]::ParameterValue, 'Once this is done, then that (a new parent above the target)')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'A sibling of the target, right after it')
            [CompletionResult]::new('done', 'done', [CompletionResultType]::ParameterValue, 'Finish the current task and walk back up')
            [CompletionResult]::new('drop', 'drop', [CompletionResultType]::ParameterValue, 'Abandon the current task')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Leave the current task unfinished, with a reason')
            [CompletionResult]::new('go', 'go', [CompletionResultType]::ParameterValue, 'Resume the current task; show how I got here')
            [CompletionResult]::new('why', 'why', [CompletionResultType]::ParameterValue, 'Chain of notes from the root to the current task')
            [CompletionResult]::new('note', 'note', [CompletionResultType]::ParameterValue, 'Read or add note lines on a task')
            [CompletionResult]::new('rename', 'rename', [CompletionResultType]::ParameterValue, 'Change a task''s text')
            [CompletionResult]::new('pick', 'pick', [CompletionResultType]::ParameterValue, 'Choose the current task')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'The whole task tree of the active goal')
            [CompletionResult]::new('undo', 'undo', [CompletionResultType]::ParameterValue, 'Undo the last N changes')
            [CompletionResult]::new('goal', 'goal', [CompletionResultType]::ParameterValue, 'Goals: show, list, new, switch, done, rename, reopen')
            [CompletionResult]::new('jot', 'jot', [CompletionResultType]::ParameterValue, 'Quick capture to the inbox')
            [CompletionResult]::new('inbox', 'inbox', [CompletionResultType]::ParameterValue, 'View the inbox, take or drop items')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'What got finished, by day')
            [CompletionResult]::new('at', 'at', [CompletionResultType]::ParameterValue, 'Schedule a task')
            [CompletionResult]::new('agenda', 'agenda', [CompletionResultType]::ParameterValue, 'Scheduled tasks across all goals')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Live task list in a terminal pane')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Where the task list lives')
            [CompletionResult]::new('prompt', 'prompt', [CompletionResultType]::ParameterValue, 'One short line for a shell prompt')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Print shell completion script')
            break
        }
        'trk;need' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Supply the why inline (skips the prompt)')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'Supply the why inline (skips the prompt)')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Pick the target task instead of using the current task')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'Pick the target task instead of using the current task')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Do not move the cursor onto the new task')
            [CompletionResult]::new('--stay', '--stay', [CompletionResultType]::ParameterName, 'Do not move the cursor onto the new task')
            [CompletionResult]::new('--no-why', '--no-why', [CompletionResultType]::ParameterName, 'Do not ask for a why')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;also' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'w')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'why')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('--no-why', '--no-why', [CompletionResultType]::ParameterName, 'no-why')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;then' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'w')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'why')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'pick')
            [CompletionResult]::new('--no-why', '--no-why', [CompletionResultType]::ParameterName, 'no-why')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;add' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'w')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'why')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'pick')
            [CompletionResult]::new('--no-why', '--no-why', [CompletionResultType]::ParameterName, 'no-why')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;done' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Finish even if the task has open sub-tasks')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'Finish even if the task has open sub-tasks')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;drop' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'w')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'why')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;stop' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'pick')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;go' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;why' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'pick')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;note' {
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'r')
            [CompletionResult]::new('--replace', '--replace', [CompletionResultType]::ParameterName, 'replace')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'c')
            [CompletionResult]::new('--clear', '--clear', [CompletionResultType]::ParameterName, 'clear')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'pick')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;rename' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'pick')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;pick' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;list' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'a')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'all')
            [CompletionResult]::new('--all-goals', '--all-goals', [CompletionResultType]::ParameterName, 'all-goals')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;undo' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;goal' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Print title, id, active marker, and counts')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'Numbered list of goals')
            [CompletionResult]::new('new', 'new', [CompletionResultType]::ParameterValue, 'Create an open goal and make it active')
            [CompletionResult]::new('switch', 'switch', [CompletionResultType]::ParameterValue, 'Make another open goal active')
            [CompletionResult]::new('done', 'done', [CompletionResultType]::ParameterValue, 'Mark the active goal done')
            [CompletionResult]::new('rename', 'rename', [CompletionResultType]::ParameterValue, 'Rename the active goal')
            [CompletionResult]::new('reopen', 'reopen', [CompletionResultType]::ParameterValue, 'Reopen a done goal and make it active')
            break
        }
        'trk;goal;show' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;goal;list' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;goal;new' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'n')
            [CompletionResult]::new('--stay', '--stay', [CompletionResultType]::ParameterName, 'stay')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;goal;switch' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;goal;done' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'f')
            [CompletionResult]::new('--force', '--force', [CompletionResultType]::ParameterName, 'force')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;goal;rename' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;goal;reopen' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;jot' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;inbox' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('take', 'take', [CompletionResultType]::ParameterValue, 'Take an item out of the inbox into a goal')
            [CompletionResult]::new('drop', 'drop', [CompletionResultType]::ParameterValue, 'Drop an item from the inbox')
            break
        }
        'trk;inbox;take' {
            [CompletionResult]::new('--goal', '--goal', [CompletionResultType]::ParameterName, 'goal')
            [CompletionResult]::new('--new-goal', '--new-goal', [CompletionResultType]::ParameterName, 'new-goal')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'pick')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 's')
            [CompletionResult]::new('--switch', '--switch', [CompletionResultType]::ParameterName, 'switch')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;inbox;drop' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;log' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;at' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'p')
            [CompletionResult]::new('--pick', '--pick', [CompletionResultType]::ParameterName, 'pick')
            [CompletionResult]::new('--clear', '--clear', [CompletionResultType]::ParameterName, 'Remove the schedule')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;agenda' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;start' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('--focus', '--focus', [CompletionResultType]::ParameterName, 'Start in focus view')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;config' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Print effective values')
            [CompletionResult]::new('path', 'path', [CompletionResultType]::ParameterValue, 'Print only the config file path')
            break
        }
        'trk;config;show' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;config;path' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;prompt' {
            [CompletionResult]::new('--max', '--max', [CompletionResultType]::ParameterName, 'max')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;completions' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
