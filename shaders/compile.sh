#!/bin/sh

SCRIPTPATH="$( cd "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

glslc $SCRIPTPATH/shader.vert -o $SCRIPTPATH/vert.spv
glslc $SCRIPTPATH/shader.frag -o $SCRIPTPATH/frag.spv
