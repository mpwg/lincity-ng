#!/usr/bin/env bash

zcat | sed '3,$y/ /-/' |
  xsltproc "$(dirname -- "$0")/1328-to-2130.xsl" - |
  gzip -9 -c
