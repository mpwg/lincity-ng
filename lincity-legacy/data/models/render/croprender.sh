#!/bin/sh

cd "$(dirname "$0")" || exit

for i in ../../images/tiles.render/*; do
    ./autocrop "$i" "$i"
done

# EOF #
