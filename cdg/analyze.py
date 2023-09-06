# coding: utf-8
# ToDo: Markowitz model analysis, getting portfolio as input and returning ideal portfolio data or difference if a set distribution was inputed

from cdg.main import *
from cdg.get import *
from pandas_datareader import data as wb


# Setup



# Funcs

def port(port=None, currency='usd', from_time=None, to_time=None, bench=True):
    """
    Gather data from a list of coins and store it for further analysis and/or plotting.

    :param port: str
    :param currency: str
    :param from_time: timestamp
    :param to_time: timestamp
    :param bench: boolean
    :return:
    """
    get_time()
    data = {}
    global analyzed_port
    if port is None:
        port = ['bitcoin', 'ethereum', 'binancecoin']
    for coin in port:
        value = get_coin_hist_by_range(coin, currency, from_time, to_time).iloc[:, :2]
        price = pd.Series(value.iloc[:, 1])
        index = pd.to_datetime(value.iloc[:, 0], unit='ms')
        price.index = index
        data[coin] = price
    df = pd.DataFrame.from_dict(data)
    if bench is True:
        bench_data = {}
        bench_tickers = ['^SP500BDT', '^GSPC', '^DJI', '^IXIC', '^HSI', '^BVSP']
        for ticker in bench_tickers:
            bench_data[ticker] = wb.DataReader(ticker, data_source='yahoo',
                                               start=str(df.index[0]),
                                               end=str(df.index[-1]), session=session)['Adj Close']
        bench = pd.DataFrame.from_dict(bench_data)
        bench.rename(columns={'^SP500BDT':'S&P500 Bounds', '^GSPC': 'S&P500', '^DJI': 'Dow Jones', '^IXIC': 'Nasdaq', '^HSI': 'Hang Seng Index', '^BVSP': 'B3'}, inplace=True)
        df = pd.concat([df, bench], axis=1)
        df.ffill(inplace=True)
        df.bfill(inplace=True)
    smp_return = ((df / df.shift(1)) - 1) * 100
    log_return = np.log(df / df.shift(1)) * 100
    cum_return = ((df.iloc[-1] - df.iloc[0]) / df.iloc[0]) * 100
    perform_normal = round((df / df.iloc[0]) * 100, 2)
    volatility = log_return.std()
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
    return(analyzed_port)
