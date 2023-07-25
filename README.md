# sectorino

# Premise

Some games for disc-based consoles intentionally duplicate data across the disc to reduce seeks and improve loading speeds. However, compressed disc image formats, especially those designed for real-time performance requiring random accessibility, generally cannot exploit this since they independently compress fixed-size blocks of data, far smaller than the distance between blocks of duplicated data.

sectorino is a experimental tool for finding blocks of duplicated data across long distances. Specifically, it breaks the input disc image into fixed-size blocks, hashes each block, and uses a rolling hash to find duplicates at arbitrary locations (across the entire image unaligned with block boundaries). It is not useful on its own and is not currently useful with anything else.

This could potentially be used as a basis for a future compressed disc image format. One option would be to remap a duplicate block (instead of storing it) to some other block plus an offset.