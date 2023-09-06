# coding: utf-8

import datetime
import requests_cache
import pandas as pd
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
def check_cached():
# ToDo: 
    session = requests_cache.CachedSession(cache_name=f'{files_path}/ycache', backend='sqlite', expire_after=expire_cache)
    session.headers = DEFAULT_HEADERS

def df_float_precision(val='sn'):
    """
    Change how pandas represents float numbers(default = scientific notation or how much numbers after the decimal).

    :param val: str or int
    :return:
    """
    if val == 'sn':
        pd.reset_option('display.float_format', silent=True)
    elif type(val) == 'int':
        pd.set_option('display.float_format', lambda x: str(f'%.{val}f') % x)
    else:
        return('error')