#!/bin/bash
# ADDS cargo-make completion support
function _cargo_make_comp() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    COMPREPLY=()
    if [[ "${COMP_WORDS[1]}" == "make" ]]; then
        if [ -f "Makefile.toml" ]; then
            while IFS='' read -r LINE; do 
                COMPREPLY+=("${LINE}");
            done < <( grep -i '\[tasks.*' Makefile.toml 2>/dev/null | sed 's/\[tasks\.\(.*\)\]/\1/ig' | grep -i "^${cur}" )
        fi
    fi
    return 0;
}

complete -F _cargo_make_comp cargo

function goto_rust(){
    local PROJECT="$1";
    if [ "${PROJECT}" = "" ]; then
        PROJECT='';
    fi

    cd ~/Documents/langs/rust/"${PROJECT}"
}
function _goto_rust_comp(){
    local cur="${COMP_WORDS[COMP_CWORD]}";
    local dir="$HOME/Documents/langs/rust/";
    COMPREPLY=()
    while IFS='' read -r LINE; do 
        COMPREPLY+=("${LINE}");
    done < <(find ${dir} -mindepth 1 -maxdepth 1 -type d -iname "${cur}*" -printf '%P\n'); 
}

complete -F _goto_rust_comp goto_rust


function enter_vps_blackbot(){
	ssh root@192.206.141.15
}

function docker_clean_everything(){
	echo 'Stop all running containers'
	docker ps -q | xargs -r docker stop
	
	echo 'Remove all containers'
	docker ps -aq | xargs -r docker rm

	echo 'Remove all images'
	docker images -q | xargs -r docker rmi

	echo 'Remove all volumes'
	docker volume ls -q | xargs -r docker volume rm

	echo 'Remove all unused networks'
	docker network ls -q | grep -v '^bridge$\|^host$\|^none$' | xargs -r docker network rm
}

function deepseek_harness(){
    local CUR_CWD="$CWD";
    cd ~/Documents/langs/js/deepseek-harness
    pnpm dsh web &
    cd "$CUR_CWD"
}

