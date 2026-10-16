
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
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a task under the goal')
            [CompletionResult]::new('then', 'then', [CompletionResultType]::ParameterValue, 'Add a task as a parent of the most recently added task')
            [CompletionResult]::new('and', 'and', [CompletionResultType]::ParameterValue, 'Add a task as a sibling of the most recently added task')
            [CompletionResult]::new('before', 'before', [CompletionResultType]::ParameterValue, 'Add a task as a sibling before the most recently added task')
            [CompletionResult]::new('done', 'done', [CompletionResultType]::ParameterValue, 'Complete the current task')
            [CompletionResult]::new('time', 'time', [CompletionResultType]::ParameterValue, 'Start timing the current task')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop timing the current task')
            [CompletionResult]::new('drop', 'drop', [CompletionResultType]::ParameterValue, 'Delete a task, using the picker')
            [CompletionResult]::new('switch', 'switch', [CompletionResultType]::ParameterValue, 'Switch tasks using the picker')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'Show the current task, its siblings, and its parents')
            [CompletionResult]::new('undo', 'undo', [CompletionResultType]::ParameterValue, 'Undo the last N changes')
            [CompletionResult]::new('new', 'new', [CompletionResultType]::ParameterValue, 'Create a goal')
            [CompletionResult]::new('progress', 'progress', [CompletionResultType]::ParameterValue, 'Show the whole goal as a tree with progress')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete a goal and every task under it')
            [CompletionResult]::new('jot', 'jot', [CompletionResultType]::ParameterValue, 'Capture a task in the inbox')
            [CompletionResult]::new('inbox', 'inbox', [CompletionResultType]::ParameterValue, 'Open the inbox in the picker')
            [CompletionResult]::new('log', 'log', [CompletionResultType]::ParameterValue, 'What got finished, by day')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Install shell completions for the detected shell')
            break
        }
        'trk;add' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Explain why this task exists (skips the prompt)')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'Explain why this task exists (skips the prompt)')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('--no-why', '--no-why', [CompletionResultType]::ParameterName, 'Do not ask for a why')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;then' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Explain why this task exists (skips the prompt)')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'Explain why this task exists (skips the prompt)')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Wrap the current task instead of the most recently added one')
            [CompletionResult]::new('--next', '--next', [CompletionResultType]::ParameterName, 'Wrap the current task instead of the most recently added one')
            [CompletionResult]::new('--no-why', '--no-why', [CompletionResultType]::ParameterName, 'Do not ask for a why')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;and' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Explain why this task exists (skips the prompt)')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'Explain why this task exists (skips the prompt)')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('--no-why', '--no-why', [CompletionResultType]::ParameterName, 'Do not ask for a why')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;before' {
            [CompletionResult]::new('-w', '-w', [CompletionResultType]::ParameterName, 'Explain why this task exists (skips the prompt)')
            [CompletionResult]::new('--why', '--why', [CompletionResultType]::ParameterName, 'Explain why this task exists (skips the prompt)')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('--no-why', '--no-why', [CompletionResultType]::ParameterName, 'Do not ask for a why')
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
        'trk;time' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;stop' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;drop' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Skip the confirmation prompt when the task has sub-tasks')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Skip the confirmation prompt when the task has sub-tasks')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;switch' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('goal', 'goal', [CompletionResultType]::ParameterValue, 'Switch goals using the picker')
            break
        }
        'trk;switch;goal' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;list' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('goal', 'goal', [CompletionResultType]::ParameterValue, 'List all goals')
            break
        }
        'trk;list;goal' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
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
        'trk;new' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('goal', 'goal', [CompletionResultType]::ParameterValue, 'Create a new goal and make it active')
            break
        }
        'trk;new;goal' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;progress' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('goal', 'goal', [CompletionResultType]::ParameterValue, 'All tasks in this goal as a single tree')
            break
        }
        'trk;progress;goal' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'trk;delete' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('goal', 'goal', [CompletionResultType]::ParameterValue, 'Delete a goal and all of its tasks')
            break
        }
        'trk;delete;goal' {
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'When to colorize output')
            [CompletionResult]::new('-y', '-y', [CompletionResultType]::ParameterName, 'Skip the confirmation prompt')
            [CompletionResult]::new('--yes', '--yes', [CompletionResultType]::ParameterName, 'Skip the confirmation prompt')
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
            break
        }
        'trk;log' {
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
