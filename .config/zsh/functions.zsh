# Automatically load all custom function definitions
local func_dir="${0:A:h}/functions"
if [[ -d "$func_dir" ]]; then
  for func_file in "$func_dir"/*.zsh; do
    [[ -f "$func_file" ]] && source "$func_file"
  done
fi
