alias -s {yml,yaml,ts,json,js,vim,rc}=nvim

alias pls='sudo -E env "PATH=$PATH"'

alias v="nvim"
alias lv="NVIM_APPNAME=lazyvim nvim"

alias D="run dev"
alias B="run build"
alias T="run test"

alias P="git push"
alias p="git pull"

alias lt="tree -L 2 --filelimit 150 --dirsfirst"
alias ll="ls -lah"

alias d="docker"

alias dots="git --git-dir=$HOME/.dotfiles/ --work-tree=$HOME"
alias lazydots="lazygit --git-dir=$HOME/.dotfiles --work-tree=$HOME"
alias pls='sudo -E env "PATH=$PATH"'

# Power Profile and Keep-Awake aliases
alias keep-awake="nohup systemd-inhibit --what=idle --who=\"manual\" --why=\"Manual toggle\" wayinhibit >/dev/null 2>&1 &"
alias keep-awake-stop="pkill wayinhibit"
alias power-saver="powerprofilesctl set power-saver"
alias power-balanced="powerprofilesctl set balanced"
alias power-performance="powerprofilesctl set performance"
