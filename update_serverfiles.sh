abs_path=$(realpath "serverfiles")
steamcmd +force_install_dir $abs_path +login anonymous +app_update 2394010 validate +quit
