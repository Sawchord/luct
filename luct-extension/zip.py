#!/usr/bin/env python3
# Source: https://fekir.info/post/reproducible-zip-archives/

import os
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED, ZIP_STORED

if __name__ == '__main__':
    zipf = ZipFile('luct.xpi', 'w', ZIP_DEFLATED) 
    path = 'luct'

    for root, dirs, files in os.walk('luct'):
        # for d in sorted(dirs):
        #     info = ZipInfo(
        #         filename=os.path.relpath(os.path.join(root, d),path) + "/",
        #         date_time=(1980, 1, 1, 12, 1, 0)
        #         )
        #     info.external_attr = 0o40755  << 16 | 0x010
        #     info.create_system = 3
        #     info.compress_type = ZIP_STORED
        #     info.CRC = 0 # unclear why necessary for directories, maybe a bug?
        #     zipf.mkdir(info)
        for f in sorted(files):
            with open(os.path.join(root, f), 'rb') as data:
                info = ZipInfo(
                        filename=os.path.relpath(os.path.join(root, f),path),
                        date_time=(1980, 1, 1, 12, 1, 0)
                       )
                info.external_attr = 0o100644 << 16
                info.create_system = 3 # unx=3 vs fat=0
                info.compress_type = ZIP_DEFLATED
                zipf.writestr(info, data.read())
                print("Compressed file: " + os.path.join(root, f))