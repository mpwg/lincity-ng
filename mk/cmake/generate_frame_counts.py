"""Emit animation lengths from the tile manifest for the simulation library."""

import json
import sys
import xml.etree.ElementTree as ET

counts = {}
for group in ET.parse(sys.argv[1]).getroot().findall("group"):
    name = None
    for entry in group:
        if entry.tag == "resourceID":
            name = entry.attrib["name"]
            counts[name] = 0
        elif entry.tag == "image" and name is not None:
            counts[name] += 1

for name, count in counts.items():
    print(f"    {{{json.dumps(name)}, {count}}},")
