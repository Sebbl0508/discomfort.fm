#!/bin/bash

set -xe

VENV_DIR="venv"

if [ ! -d "$VENV_DIR" ]; then
    python3 -m venv ${VENV_DIR}
fi

source ${VENV_DIR}/bin/activate

pip install --no-input -U yt-dlp pip

./discomfort-fm
