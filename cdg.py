# coding: utf-8
# TODO: Make docstring for all functions.

import datetime
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from pathlib import Path
from pycoingecko import CoinGeckoAPI
from pandas_datareader import data as wb


files_path = 'cdg_files'
path = Path(files_path)
path.mkdir(exist_ok=True)
cg = CoinGeckoAPI()
time = datetime.datetime.now().timestamp()

# Basic functions


def update_time():
    global time
    time = datetime.datetime.now().timestamp()


def save_data(file, exc=False, name='output'):
    print(file)
    if type(file) == pd.DataFrame:
        df = file
    else:
        print('Error: file is not a DataFrame!')
        return
    if not exc:
        df.to_csv(f'{files_path}/{name}.csv')
    elif exc:
        df.to_excel(f'{files_path}/{name}.xlsx')
    else:
        print('Error: weird \'exc\' argument value!')


def get_server_status():
    return cg.ping()


def get_currency_support(name='all'):
    currencies = cg.get_supported_vs_currencies()
    if name == 'all':
        return currencies
    else:
        for x in currencies:
            if name.lower() in x.lower():
                return True
            else:
                return False


def get_coin_id(name='all'):
    coins = cg.get_coins_list()
    if name == 'all':
        return coins
    else:
        for x in coins:
            if x['name'].lower() == name.lower():
                return x
            else:
                print(f'Error: coin "{name}" currently not supported.')
                return


def get_resume():
    update_time()
    global_data = cg.get_global()
    df = pd.DataFrame(global_data, columns=['active_cryptocurrencies', 'upcoming_icos',
                                            'ongoing_icos', 'ended_icos', 'markets'], index=[0])
    df.index.name = time
    return df


def get_pub_treasury_data():
    update_time()
    value_usd = {}
    btc_pub_treasury = cg.get_companies_public_treasury_by_coin_id('bitcoin')
    value_usd.update({'btc': btc_pub_treasury['total_value_usd']})
    eth_pub_treasury = cg.get_companies_public_treasury_by_coin_id('ethereum')
    value_usd.update({'eth': eth_pub_treasury['total_value_usd']})
    value_usd.update({'total': value_usd['btc'] + value_usd['eth']})
    df = pd.DataFrame(value_usd, index=[0])
    df.index.name = time
    return df


def get_total_mkt_cap():
    update_time()
    global_data = cg.get_global()
    total_market_cap = {'usd': global_data['total_market_cap']['usd'], 'btc': global_data['total_market_cap']['btc'],
                        '%change_24h': global_data["market_cap_change_percentage_24h_usd"]}
    df = pd.DataFrame(total_market_cap, index=[0])
    df.index.name = time
    return df


def get_top10_mkt_cap_coins():
    update_time()
    global_data = cg.get_global()
    df = pd.DataFrame(global_data["market_cap_percentage"], index=[0])
    df.index.name = time
    return df


def get_mkt_top100():
    update_time()
    data = cg.get_coins_markets(vs_currency='usd', per_page=100)
    df = pd.DataFrame(data, columns=['market_cap_rank', 'id', 'symbol', 'current_price', 'price_change_percentage_24h',
                                     'low_24h', 'high_24h', 'total_volume'])
    df.index.name = time
    return df


def get_pair(x='bitcoin', y='usd'):
    update_time()
    data = cg.get_price(x, y, include_market_cap='true', include_24hr_vol='true', include_24hr_change='true')
    df = pd.DataFrame.from_dict(data, orient='index')
    df.index.name = time
    if df.empty:
        print('Error: DataFrame is empty!')
        return
    return df


def get_coins(*args):
    if not args:
        print('Error: missing args!')
        return
    df = pd.DataFrame()
    c = 0
    for x in args:
        df0 = get_pair(x, 'usd')
        if c > 0:
            df = pd.concat([df, df0], axis=0)
        else:
            df = df0
        c += 1
    df.index.name = time
    if df.empty:
        print('Error: DataFrame is empty!')
        return
    return df


def get_coin_hist_data(x='bitcoin', y='usd', z=90):
    data = cg.get_coin_market_chart_by_id(x, y, z)
    prices = pd.DataFrame(data['prices'], columns=['timestamp', 'price'])
    cap = pd.DataFrame(data['market_caps'], columns=['timestamp', 'mkt_cap'])
    vol = pd.DataFrame(data['total_volumes'], columns=['timestamp', 'vol'])
    df = pd.concat([prices, cap], axis=1)
    df = pd.concat([df, vol], axis=1)
    df = df.loc[:, ~df.columns.duplicated()]
    if df.empty:
        print('Error: DataFrame is empty!')
        return
    return df


def get_coin_hist_data_ohlc(x='bitcoin', y='usd', z=90):
    # Possible values to z = 1/7/14/30/90/180/365/max
    update_time()
    df = pd.DataFrame()
    data = cg.get_coin_ohlc_by_id(x, y, z)
    c = 0
    for x in data:
        ohlc = {'timestamp': [x[0]], 'opn': [x[1]], 'high': [x[2]], 'low': [x[3]], 'close': [x[4]]}
        df0 = pd.DataFrame.from_dict(ohlc, orient='columns')
        if c > 0:
            df = pd.concat([df, df0], axis=0)
        else:
            df = df0
        c += 1
    df = df.set_index(['timestamp'])
    df.reset_index(inplace=True)
    return df


def get_trending():
    update_time()
    data = cg.get_search_trending()
    df = pd.DataFrame()
    c = 0
    for _ in data['coins']:
        df0 = pd.DataFrame(data['coins'][c]['item'], columns=['id', 'symbol', 'market_cap_rank'], index=[0])
        df0['index'] = df0['id']
        df0 = df0.set_index(['index'])
        df1 = pd.DataFrame.from_dict(cg.get_price(data['coins'][c]['item']['id'], 'usd', include_market_cap='true',
                                                  include_24hr_vol='true', include_24hr_change='true'), orient='index')
        df0 = df0.join(df1)
        if c > 0:
            df = pd.concat([df, df0], axis=0)
        elif c == 0:
            df = df0
        else:
            print('Error: weird counting variable value!')
            return
        c += 1
    df.reset_index(inplace=True)
    df.index.name = time
    return df


def get_defi_mkt():
    update_time()
    df = pd.DataFrame.from_dict(cg.get_global_decentralized_finance_defi(), orient='index', columns=['Value'])
    df.reset_index(inplace=True)
    df.index.name = time
    return df


# Analysis functions
# TODO: Some data analyze functions like 'risk/return' etc...

# Plotting functions
# TODO: Some functions to automate plotting analyzes...
