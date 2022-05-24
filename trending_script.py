#!/usr/bin/env python
# coding: utf-8

from funcs import *

"""Setup"""
# Create directory to store files
Path(files_path + str(local_time.date())).mkdir(exist_ok=True)

"""Script"""
with pd.ExcelWriter(files_path + '/%s/trending.xlsx' % str(local_time.date())) as writer:
    df = get_trending()
    df.copy().to_excel(writer, sheet_name='%s' % str(local_time.hour))
