#!/bin/sh

set -e -x

cd "$(dirname "$0")"

mkdir -p tmp/
mkdir -p ../../images/tiles.render/

for i in ../*.blend; do
    blender -b "$i"  -P render.py
    blender -b tmp/tmp.blend -a
    b=`basename $i`
    cp -v tmp/0001 "../../images/tiles.render/${b%%.blend}.png"
done

# EOF #
