# coding: utf-8

import datetime
import requests_cache
import pandas as pd
import datetime
import requests_cache
from pathlib import Path
from pandas_datareader.yahoo.headers import DEFAULT_HEADERS

# Global

# Setup

"""Prep workspace"""
files_path = 'cdg_files'
path = Path(files_path)
path.mkdir(exist_ok=True)
"""Cache"""
expire_cache = datetime.timedelta(minutes=5)
session = requests_cache.CachedSession(cache_name=f'{files_path}/ycache', backend='sqlite', expire_after=expire_cache)
session.headers = DEFAULT_HEADERS
"""Time"""
date = datetime.datetime.now().strftime('%Y-%m-%d')
time = datetime.datetime.now().strftime('%H-%M-%S')
"""Global variables storing value to be manipulated"""
analyzed_port = {}
mark_port = {}

# Funcs
def clear_cache():
# ToDo: 
    session = requests_cache.CachedSession()
    session.headers = DEFAULT_HEADERS
