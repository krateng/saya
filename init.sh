#!/bin/sh

# permission checks
if ! [ -r /config ]; then
	echo "Config file not readable!"
	exit 1
else
	echo "Config file readable."
fi
if ! [ -w ./Pal/Saved/ ]; then
	echo "Saved directory is not writable!"
	exit 1
else
	echo "Saved directory writable"
fi


# generate config files
python3 generate_config.py || { echo "Failed to generate configuration file!" ; exit 1; }
echo "Generated configuration file"

# run proxy

exec socat -T15 UDP6-LISTEN:8211,fork,reuseaddr UDP4:127.0.01:${INTERNAL_SERVER_PORT} &
echo "Running proxy from 8211 to ${INTERNAL_SERVER_PORT}"

# run server
if [ "${COMMUNITY_SERVER}" = "1" ]; then
	./PalServer.sh -port=${INTERNAL_SERVER_PORT} -useperfthreads -NoAsyncLoadingThread -UseMultithreadForDS -publiclobby
else
	./PalServer.sh -port=${INTERNAL_SERVER_PORT} -useperfthreads -NoAsyncLoadingThread -UseMultithreadForDS
fi

