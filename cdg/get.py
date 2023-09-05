# coding: utf-8
# ToDo: Implement caching for coingecko API
# ToDo: Change usage of pandas to numpy when data need to will be calculated but not plotted.

import datetime
import requests_cache
import numpy as np
import pandas as pd
from pathlib import Path
from dateutil.relativedelta import relativedelta
from pycoingecko import CoinGeckoAPI
from pandas_datareader.yahoo.headers import DEFAULT_HEADERS

# Setup

"""Prep workspace"""
files_path = 'cdg_files'
path = Path(files_path)
path.mkdir(exist_ok=True)
cg = CoinGeckoAPI()
"""Cache"""
expire_cache = datetime.timedelta(minutes=5)
session = requests_cache.CachedSession(cache_name=f'{files_path}/ycache', backend='sqlite', expire_after=expire_cache)
session.headers = DEFAULT_HEADERS
"""Time"""
date = datetime.datetime.now().strftime('%Y-%m-%d')
time = datetime.datetime.now().strftime('%H-%M-%S')
"""Global variables storing value to be manipulated"""
analyzed_port = {}

# Funcs

def get_time():
    """
    Update date and time values.

    :return:
    """
    global date, time
    date = datetime.datetime.now().strftime('%Y-%m-%d')
    time = datetime.datetime.now().strftime('%H-%M-%S')

def get_server_status():
    """
    Ping coingecko API.

    :return: str
    """
    with requests_cache.disabled():
        get_time()
        return f'{cg.ping()} AT {time} FROM {date}'

def get_currency_support(name='all'):
    """
    Get a list of supported currencies or check if specific currency is supported.

    :param name: str
    :return: dict or boolean
    """
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
    """
    Get a list of supported coins or check if specific coin is supported.

    :param name: str
    :return: str
    """
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
    """
    Get a resume of today's crypto market.

    :return: DataFrame
    """
    get_time()
    global_data = cg.get_global()
    df = pd.DataFrame(global_data, columns=['active_cryptocurrencies', 'upcoming_icos',
                                            'ongoing_icos', 'ended_icos', 'markets'], index=[0])
    df.index.name = f'{date}_{time}'
    return df

def get_pub_treasury_data():
    """
    Get institutional holdings of BTC and ETH values.

    :return: DataFrame
    """
    get_time()
    value_usd = {}
    btc_pub_treasury = cg.get_companies_public_treasury_by_coin_id('bitcoin')
    value_usd.update({'btc': btc_pub_treasury['total_value_usd']})
    eth_pub_treasury = cg.get_companies_public_treasury_by_coin_id('ethereum')
    value_usd.update({'eth': eth_pub_treasury['total_value_usd']})
    value_usd.update({'total': value_usd['btc'] + value_usd['eth']})
    df = pd.DataFrame(value_usd, index=[0])
    df.index.name = f'{date}_{time}'
    return df

def get_total_mkt_cap():
    """
    Get market cap values in USD and BTC, and percentage of change in the last 24H.

    :return: DataFrame
    """
    get_time()
    global_data = cg.get_global()
    total_market_cap = {'usd': global_data['total_market_cap']['usd'], 'btc': global_data['total_market_cap']['btc'],
                        '%change_24h': global_data["market_cap_change_percentage_24h_usd"]}
    df = pd.DataFrame(total_market_cap, index=[0])
    df.index.name = f'{date}_{time}'
    return df

def get_top10_mkt_cap_coins():
    """
    Get the top10 coins by market cap.

    :return: DataFrame
    """
    get_time()
    global_data = cg.get_global()
    df = pd.DataFrame(global_data["market_cap_percentage"], index=[0])
    df.index.name = f'{date}_{time}'
    return df

def get_mkt_top100():
    """
    Get a list of the top100 coins.

    :return: DataFrame
    """
    get_time()
    data = cg.get_coins_markets(vs_currency='usd', per_page=100)
    df = pd.DataFrame(data, columns=['market_cap_rank', 'id', 'symbol', 'current_price', 'price_change_percentage_24h',
                                     'low_24h', 'high_24h', 'total_volume'])
    df.index.name = f'{date}_{time}'
    return df

def get_pair(coin='bitcoin', currency='usd'):
    """
    Get a pair (coin vs currency) market value and volume.

    :param coin: str
    :param currency: str
    :return: DataFrame
    """
    get_time()
    data = cg.get_price(coin, currency, include_market_cap='true', include_24hr_vol='true', include_24hr_change='true')
    df = pd.DataFrame.from_dict(data, orient='index')
    df.index.name = f'{date}_{time}'
    if df.empty:
        print('Error: DataFrame is empty!')
        return
    return df

def get_coins(*args):
    """
    Get a list of pairs (coins vs usd) market values and volume.

    :param args: str
    :return: DataFrame
    """
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
    df.index.name = f'{date}_{time}'
    if df.empty:
        print('Error: DataFrame is empty!')
        return
    return df

def get_coin_hist(coin='bitcoin', currency='usd', timeframe=90):
    """
    Get coin historical values and volume.

    Auto granularity.

    :param coin: str
    :param currency: str
    :param timeframe: int
    :return: DataFrame
    """
    data = cg.get_coin_market_chart_by_id(coin, currency, timeframe)
    prices = pd.DataFrame(data['prices'], columns=['timestamp', 'price'])
    cap = pd.DataFrame(data['market_caps'], columns=['timestamp', 'mkt_cap'])
    vol = pd.DataFrame(data['total_volumes'], columns=['timestamp', 'vol'])
    df = pd.concat([prices, cap, vol], axis=1)
    df = df.loc[:, ~df.columns.duplicated()]
    if df.empty:
        print('Error: DataFrame is empty!')
        return
    return df

def get_coin_hist_by_range(coin='bitcoin', currency='usd', from_time=None, to_time=None):
    """
    Get coin historical values and volume in a defined timerange.

    Auto granularity.

    :param coin: str
    :param currency: str
    :param from_time: timestamp
    :param to_time: timestamp
    :return: DataFrame
    """
    if from_time is None:
        from_time = (datetime.datetime.now() - relativedelta(months=4)).timestamp()
    if to_time is None:
        to_time = (datetime.datetime.now() - relativedelta(days=1)).timestamp()
    data = cg.get_coin_market_chart_range_by_id(coin, currency, from_time, to_time)
    prices = pd.DataFrame(data['prices'], columns=['timestamp', 'price'])
    cap = pd.DataFrame(data['market_caps'], columns=['timestamp', 'mkt_cap'])
    vol = pd.DataFrame(data['total_volumes'], columns=['timestamp', 'vol'])
    df = pd.concat([prices, cap, vol], axis=1)
    df = df.loc[:, ~df.columns.duplicated()]
    if df.empty:
        print('Error: DataFrame is empty!')
        return
    return df

def get_coin_hist_ohlc(coin='bitcoin', currency='usd', timeframe=90):
    """
    Get coin historical OHLC.

    Auto granularity.

    Possible timeframe values = 1/7/14/30/90/180/365/"max".

    :param coin: str
    :param currency: str
    :param timeframe: int
    :return: DataFrame
    """
    get_time()
    df = pd.DataFrame()
    data = cg.get_coin_ohlc_by_id(coin, currency, timeframe)
    c = 0
    for coin in data:
        ohlc = {'timestamp': [coin[0]], 'opn': [coin[1]], 'high': [coin[2]], 'low': [coin[3]], 'close': [coin[4]]}
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
    """
    Get a list with the current trending coins.

    :return: DataFrame
    """
    get_time()
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
    df.index.name = f'{date}_{time}'
    return df

def get_defi_mkt():
    """
    Get a resume about DeFi markets.

    :return: DataFrame
    """
    get_time()
    df = pd.DataFrame.from_dict(cg.get_global_decentralized_finance_defi(), orient='index', columns=['Value'])
    df.reset_index(inplace=True)
    df.index.name = f'{date}_{time}'
    return df
