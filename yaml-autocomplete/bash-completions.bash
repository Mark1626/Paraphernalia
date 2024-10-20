#!/usr/bin/env bash

CMD_OPT="run diagnose"
RUN_OPT="--path --set"

function _run_cmd() {
    local curr prev
    prev=$1
    curr=$2
    case ${prev} in
        --set )
            keys=($(yq --output-format props scratch.yml | sed 's/^//; s/ =.*$//g'));
            COMPREPLY=( $(egrep -o "${curr//./\\.}[^ ]+" <<< ${keys[@]}) );
            ;;
        --path )
            COMPREPLY=()
            ;;
        * )
            COMPREPLY=( $(compgen -W "${RUN_OPT}" -- ${curr}) )
            ;;
    esac
}

function _test() {
    local curr prev
    COMPREPLY=()

    curr=${COMP_WORDS[COMP_CWORD]}
    prev=${COMP_WORDS[COMP_CWORD-1]}
    subcommand=${COMP_WORDS[1]}

    case ${COMP_CWORD} in
        1 )
            COMPREPLY=( $(compgen -W "${CMD_OPT}" -- ${curr}) )
        ;;
        * )
            case ${subcommand} in
                run )
                    _run_cmd $prev $curr
                ;;
            esac
        ;;
    esac
}

complete -o bashdefault -o default -o nospace -F _test test
