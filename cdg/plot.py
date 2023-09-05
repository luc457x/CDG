# coding: utf-8

from cdg.get import *
import matplotlib.pyplot as plt
import seaborn as sns

# Setup

sns.set_theme(context='talk', style='darkgrid', palette='dark', font='dejavu serif')

# Funcs

def change_float_precision(val='sn'):
    """
    Change how float numbers are represented (default scientific notation or how much numbers after the decimal).

    :param val: str or int
    :return:
    """
    if val == 'sn':
        pd.reset_option('display.float_format', silent=True)
    else:
        pd.set_option('display.float_format', lambda x: str(f'%.{val}f') % x)


def plot_set_theme(theme='dark'):
    """
    Change between plotting preset themes (dark, light or colorblind).

    :param theme: str
    :return:
    """
    if theme == 'dark':
        sns.set_theme(palette='dark')
    elif theme == 'light':
        sns.set_theme(palette='deep')
    elif theme == 'colorblind':
        sns.set_theme(palette='colorblind')


def plot_returns(x=18, y=6, close=True, log=False):
    """
    Save line-plot from returns of analysed coins data.

    :param close: bool
    :param x: int
    :param y: int
    :param log: boolean
    :return:
    """
    update_time()
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
    if close:
        plt.close()


def plot_performance(x=18, y=6, close=True):
    """
    Save line-plot from performance of analysed coins data.

    :param close: bool
    :param x: int
    :param y: int
    :return:
    """
    update_time()
    plt.figure(figsize=(x, y))
    plt.tick_params(axis='both', which='major', labelsize=14)
    plot = sns.lineplot(data=analyzed_port["perform_normal"], dashes=False)
    plot.set(title='Performance')
    plt.legend(fontsize='14')
    plot.yaxis.set_major_formatter('{x:1.0f}%')
    plt.savefig(f'{files_path}/plot_performance_{date}_{time}.png')
    if close:
        plt.close()


def plot_risk_return(x=18, y=6, close=True):
    """
    Save scatter-plot from risk&return of analysed coins data.

    :param close: bool
    :param x: int
    :param y: int
    :return:
    """
    update_time()
    plt.figure(figsize=(x, y))
    plt.tick_params(axis='both', which='major', labelsize=14)
    df = pd.concat([analyzed_port["volatility"], round(analyzed_port['log_return'].mean() * 100, 2)], axis=1)
    df.reset_index(inplace=True)
    df.columns = ['index', 'Risk', 'Return']
    plot = sns.scatterplot(data=df, x='Risk', y='Return', hue='index')
    plot.set(title='Risk/Return')
    plt.legend(fontsize='14')
    plt.savefig(f'{files_path}/risk&return_{date}_{time}.png')
    if close:
        plt.close()
