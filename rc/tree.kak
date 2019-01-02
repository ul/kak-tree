# Path to the kak-tree executable.
# To load config:          set-option global tree_cmd "kak-tree --config /path/to/kak-tree.toml"
# To enable debug logging: set-option global tree_cmd "kak-tree -vvv"
declare-option str tree_cmd "kak-tree"

# Path to the log file.
declare-option str tree_log "/tmp/kak-tree.log"

# Option to store draft of the current buffer before passing to shell.
declare-option -hidden str tree_draft

define-command -hidden tree-command -params 1..2 -docstring %{
    tree-command <OP_TYPE> [<OP_PARAMS>]
    Send request to kak-tree and evaluate response.
} %{
    evaluate-commands -draft -no-hooks %{exec '%'; set buffer tree_draft %val{selection}}
    evaluate-commands %sh{

tree_draft=$(printf '%s.' "${kak_opt_tree_draft}" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | sed "s/$(printf '\t')/\\\\t/g")

tree_draft=${tree_draft%.}

printf '
filetype  = "%s"
selections_desc = "%s"
content = """
%s"""
[op]
type = "%s"
%s
' "${kak_opt_filetype}" "${kak_selections_desc}" "${tree_draft}" $1 "$2" | ${kak_opt_tree_cmd} 2>${kak_opt_tree_log}
    }
}


define-command -hidden tree-command-with-optional-kind -params 1..2 -docstring %{
    tree-command-with-optional-kind <OP_TYPE> [<KIND>]
    Send request which optionally takes node kind.
} %{
    tree-command %arg{1} %sh{
        if [ -n "$2" ]; then
            printf 'kind = "%s"' "$2"
        fi
    }
}

define-command tree-select-parent-node -params ..1 -docstring %{
    tree-select-parent-node [<KIND>]
    Select the closest visible ancestor or ancestor of KIND when provided.
} %{ tree-command-with-optional-kind SelectParentNode %arg{1} }

define-command tree-select-next-node -params ..1 -docstring %{
    tree-select-next-node [<KIND>]
    Select the closest visible next sibling or next sibling of KIND when provided.
} %{ tree-command-with-optional-kind SelectNextNode %arg{1} }

define-command tree-select-previous-node -params ..1 -docstring %{
    tree-select-previous-node [<KIND>]
    Select the closest visible previous sibling or previous sibling of KIND when provided.
} %{ tree-command-with-optional-kind SelectPreviousNode %arg{1} }

define-command tree-select-children -params ..1 -docstring %{
    tree-select-children [<KIND>]
    Select all immediate visible children or all descendants matching KIND when provided.
} %{ tree-command-with-optional-kind SelectChildren %arg{1} }

define-command tree-node-sexp -docstring %{
    tree-node-sexp
    Show info box with a syntax tree of the main selection parent.
} %{ tree-command NodeSExp }

define-command tree-select-first-child -params ..1 -docstring %{
    tree-select-first-child [<KIND>]
    Select the first immediate visible children or the first descendant matching KIND when provided.
} %{
    tree-command-with-optional-kind SelectChildren %arg{1}
    execute-keys <space>
}

