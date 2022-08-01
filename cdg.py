# coding: utf-8
# ToDo: Make docstring for all functions.

import datetime
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from pathlib import Path
from dateutil.relativedelta import relativedelta
from pycoingecko import CoinGeckoAPI
from pandas_datareader import data as wb

# Setup

files_path = 'cdg_files'
path = Path(files_path)
path.mkdir(exist_ok=True)
cg = CoinGeckoAPI()
date = datetime.datetime.now().strftime('%Y-%m-%d')
time = datetime.datetime.now().strftime('%H-%M-%S')
analyzed_port = {}
sns.set_theme(context='talk', style='darkgrid', palette='dark', font='dejavu serif')


def update_time():
    global date, time
    date = datetime.datetime.now().strftime('%Y-%m-%d')
    time = datetime.datetime.now().strftime('%H-%M-%S')


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
    df.index.name = f'{date}_{time}'
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
    df.index.name = f'{date}_{time}'
    return df


def get_total_mkt_cap():
    update_time()
    global_data = cg.get_global()
    total_market_cap = {'usd': global_data['total_market_cap']['usd'], 'btc': global_data['total_market_cap']['btc'],
                        '%change_24h': global_data["market_cap_change_percentage_24h_usd"]}
    df = pd.DataFrame(total_market_cap, index=[0])
    df.index.name = f'{date}_{time}'
    return df


def get_top10_mkt_cap_coins():
    update_time()
    global_data = cg.get_global()
    df = pd.DataFrame(global_data["market_cap_percentage"], index=[0])
    df.index.name = f'{date}_{time}'
    return df


def get_mkt_top100():
    update_time()
    data = cg.get_coins_markets(vs_currency='usd', per_page=100)
    df = pd.DataFrame(data, columns=['market_cap_rank', 'id', 'symbol', 'current_price', 'price_change_percentage_24h',
                                     'low_24h', 'high_24h', 'total_volume'])
    df.index.name = f'{date}_{time}'
    return df


def get_pair(coin='bitcoin', currency='usd'):
    update_time()
    data = cg.get_price(coin, currency, include_market_cap='true', include_24hr_vol='true', include_24hr_change='true')
    df = pd.DataFrame.from_dict(data, orient='index')
    df.index.name = f'{date}_{time}'
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
    df.index.name = f'{date}_{time}'
    if df.empty:
        print('Error: DataFrame is empty!')
        return
    return df


def get_coin_hist(coin='bitcoin', currency='usd', timeframe=90):
    data = cg.get_coin_market_chart_by_id(coin, currency, timeframe)
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


def get_coin_hist_by_range(coin='bitcoin', currency='usd', from_time=None, to_time=None):
    if from_time is None:
        from_time = (datetime.datetime.now() - relativedelta(months=4)).timestamp()
    if to_time is None:
        to_time = (datetime.datetime.now() - relativedelta(days=1)).timestamp()
    data = cg.get_coin_market_chart_range_by_id(coin, currency, from_time, to_time)
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


def get_coin_hist_ohlc(coin='bitcoin', currency='usd', timeframe=90):
    # Possible timeframe values = 1/7/14/30/90/180/365/max
    update_time()
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
    df.index.name = f'{date}_{time}'
    return df


def get_defi_mkt():
    update_time()
    df = pd.DataFrame.from_dict(cg.get_global_decentralized_finance_defi(), orient='index', columns=['Value'])
    df.reset_index(inplace=True)
    df.index.name = f'{date}_{time}'
    return df


def analyze_coins(port=None, currency='usd', from_time=None, to_time=None, bench=True):
    # 'from_time' and 'to_time' need to be a timestamp.
    global analyzed_port
    if port is None:
        port = ['bitcoin', 'ethereum', 'binancecoin']
    update_time()
    data = {}
    print('Analysing coins...')
    for coin in port:
        value = get_coin_hist_by_range(coin, currency, from_time, to_time).iloc[:, :2]
        price = pd.Series(value.iloc[:, 1])
        index = pd.to_datetime(value.iloc[:, 0], unit='ms')
        price.index = index
        data[coin] = price
    df = pd.DataFrame.from_dict(data)
    if bench is True:
        print('Getting benchmark data...')
        bench_data = {}
        bench_tickers = ['^DJI', '^GSPC', '^IXIC']
        for ticker in bench_tickers:
            bench_data[ticker] = wb.DataReader(ticker, data_source='yahoo',
                                               start=str(df.index[0]),
                                               end=str(df.index[-1]))['Adj Close']
        bench = pd.DataFrame.from_dict(bench_data)
        bench.rename(columns={'^DJI': 'dow jones', '^GSPC': 's&p500', '^IXIC': 'nasdaq'}, inplace=True)
        df = pd.concat([df, bench], axis=1)
        df.ffill(inplace=True)
        df.bfill(inplace=True)
    smp_return = ((df / df.shift(1)) - 1) * 100
    log_return = np.log(df / df.shift(1)) * 100
    cum_return = ((df.iloc[-1] - df.iloc[0]) / df.iloc[0]) * 100
    perform_normal = round((df / df.iloc[0]) * 100, 2)
    volatility = log_return.std() * 100
    analyzed_port["prices"] = df
    smp_return.dropna(how='all', inplace=True)
    smp_return.fillna(0, inplace=True)
    analyzed_port['smp_return'] = smp_return
    log_return.dropna(how='all', inplace=True)
    log_return.fillna(0, inplace=True)
    analyzed_port['log_return'] = log_return
    analyzed_port["cum_return"] = cum_return
    perform_normal.ffill(inplace=True)
    analyzed_port["perform_normal"] = perform_normal
    analyzed_port["volatility"] = volatility
    print('Analysis finished!')


def plot_set_theme(theme='dark'):
    if theme == 'dark':
        sns.set_theme(palette='dark')
    elif theme == 'light':
        sns.set_theme(palette='deep')
    elif theme == 'colorblind':
        sns.set_theme(palette='colorblind')


def plot_returns(x=18, y=6, log=False):
    if log:
        returns = "log_return"
    else:
        returns = "smp_return"
    plt.figure(figsize=(x, y))
    plt.tick_params(axis='both', which='major', labelsize=14)
    plot = sns.lineplot(data=analyzed_port[returns], dashes=False)
    plot.set(title='Return')
    plt.legend(fontsize='14')
    plot.yaxis.set_major_formatter('{x:1.0f}%')
    plt.savefig(f'{files_path}/plot_return_{date}_{time}.png')
    plt.close()


def plot_performance(x=18, y=6):
    plt.figure(figsize=(x, y))
    plt.tick_params(axis='both', which='major', labelsize=14)
    plot = sns.lineplot(data=analyzed_port["perform_normal"], dashes=False)
    plot.set(title='Performance')
    plt.legend(fontsize='14')
    plot.yaxis.set_major_formatter('{x:1.0f}%')
    plt.savefig(f'{files_path}/plot_performance_{date}_{time}.png')
    plt.close()
