#!/usr/bin/env python3
# Source: https://fekir.info/post/reproducible-zip-archives/

import os, hashlib
from zipfile import ZipFile, ZipInfo, ZIP_DEFLATED, ZIP_STORED

if __name__ == '__main__':
    zipf = ZipFile('luct.xpi', 'w', ZIP_DEFLATED) 
    zippath = 'luct'

    for root, dirs, files in sorted(os.walk('luct')):
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
            filepath = os.path.join(root, f)
            with open(filepath, 'rb') as data:
                digest = hashlib.file_digest(data, "sha256")
                data.seek(0, 0)

                info = ZipInfo(
                        filename=os.path.relpath(filepath, zippath),
                        date_time=(1980, 1, 1, 12, 1, 0)
                       )
                info.external_attr = 0o100644 << 16
                info.create_system = 3 # unx=3 vs fat=0
                info.compress_type = ZIP_DEFLATED
                zipf.writestr(info, data.read())
                print("Compressing [" + digest.hexdigest()[0:12] + "] " +  filepath)