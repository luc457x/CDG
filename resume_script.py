#!/usr/bin/env python
# coding: utf-8

from funcs import *

"""Setup"""
# Create directory to store files
Path(files_path + str(local_time.date())).mkdir(exist_ok=True)

"""Script"""
with pd.ExcelWriter(files_path + '/%s/daily_report.xlsx' % str(local_time.date())) as writer:
    df = get_total_mkt_cap()
    df.copy().to_excel(writer, sheet_name='total market cap')
    df = get_defi_mkt()
    df.copy().to_excel(writer, sheet_name='defi market cap')
    df = get_pub_treasury_data()
    df.copy().to_excel(writer, sheet_name='public treasury')
    df = get_coins('bitcoin', 'ethereum', 'binancecoin', 'chainlink', 'xrp', 'cardano', 'zilliqa', 'iotex', 'ergo')
    df.copy().to_excel(writer, sheet_name='Watchlist')
