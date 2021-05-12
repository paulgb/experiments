#!/bin/sh

set -e


if [ "$1" == "" ]
then
	echo "Usage: ./new.sh module-name"
	exit
fi

DATE=$(date +"%Y-%m-%d")
DIRECTORY="${DATE}-$1"

cargo init --name "$1" ${DIRECTORY} 

