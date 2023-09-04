autoload -U add-zsh-hook

_start() {
    local id
    id=$(./histo start --host $HOST --pwd $PWD -- $1)
    export _HISTO_ID=$id
}

_end() {
    local exit_code=$?
    [[ -z ${_HISTO_ID:-} ]] && return
    ./histo end --id $_HISTO_ID --exit-code $exit_code
}

add-zsh-hook preexec _start
add-zsh-hook precmd _end
